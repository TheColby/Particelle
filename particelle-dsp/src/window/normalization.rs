use super::schema::WindowNormalization;

/// Apply normalization scaling directly to the window slice.
pub fn apply_normalization(window_slice: &mut [f64], norm: WindowNormalization) {
    if window_slice.is_empty() || norm == WindowNormalization::None {
        return;
    }

    match norm {
        WindowNormalization::Peak => {
            let mut max = 0.0_f64;
            for &v in window_slice.iter() {
                let a = v.abs();
                if a > max {
                    max = a;
                }
            }
            if max > 0.0 && (max - 1.0).abs() > 1e-12 {
                let scale = 1.0 / max;
                for v in window_slice.iter_mut() {
                    *v *= scale;
                }
            }
        }
        WindowNormalization::Sum => {
            let sum: f64 = window_slice.iter().map(|v| v.abs()).sum();
            if sum > 0.0 {
                let scale = 1.0 / sum;
                for v in window_slice.iter_mut() {
                    *v *= scale;
                }
            }
        }
        WindowNormalization::Rms => {
            let sq_sum: f64 = window_slice.iter().map(|&v| v * v).sum();
            let rms = (sq_sum / window_slice.len() as f64).sqrt();
            if rms > 0.0 {
                let scale = 1.0 / rms;
                for v in window_slice.iter_mut() {
                    *v *= scale;
                }
            }
        }
        WindowNormalization::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_slice_eq(a: &[f64], b: &[f64], epsilon: f64) {
        assert_eq!(a.len(), b.len(), "Slices have different lengths");
        for (i, (&va, &vb)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (va - vb).abs() <= epsilon,
                "Slices differ at index {}: {} != {} (epsilon: {})",
                i, va, vb, epsilon
            );
        }
    }

    #[test]
    fn test_apply_normalization_empty_slice() {
        let mut empty: [f64; 0] = [];
        apply_normalization(&mut empty, WindowNormalization::Peak);
        assert_eq!(empty.len(), 0);

        apply_normalization(&mut empty, WindowNormalization::Sum);
        assert_eq!(empty.len(), 0);

        apply_normalization(&mut empty, WindowNormalization::Rms);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_apply_normalization_none() {
        let original = [1.0, -2.0, 3.0, -4.0, 5.0];
        let mut slice = original;
        apply_normalization(&mut slice, WindowNormalization::None);
        assert_slice_eq(&slice, &original, 1e-12);
    }

    #[test]
    fn test_apply_normalization_peak() {
        // Normal case
        let mut slice = [1.0, -2.0, 3.0, -4.0, 5.0];
        apply_normalization(&mut slice, WindowNormalization::Peak);
        assert_slice_eq(&slice, &[0.2, -0.4, 0.6, -0.8, 1.0], 1e-12);

        // All zeros case
        let mut zeros = [0.0, 0.0, 0.0];
        apply_normalization(&mut zeros, WindowNormalization::Peak);
        assert_slice_eq(&zeros, &[0.0, 0.0, 0.0], 1e-12);

        // Already normalized case
        let mut normalized = [0.5, 1.0, -0.5];
        apply_normalization(&mut normalized, WindowNormalization::Peak);
        assert_slice_eq(&normalized, &[0.5, 1.0, -0.5], 1e-12);
    }

    #[test]
    fn test_apply_normalization_sum() {
        // Normal case
        let mut slice = [1.0, -2.0, 3.0, -4.0]; // sum of abs = 1+2+3+4 = 10
        apply_normalization(&mut slice, WindowNormalization::Sum);
        assert_slice_eq(&slice, &[0.1, -0.2, 0.3, -0.4], 1e-12);
        let sum: f64 = slice.iter().map(|v| v.abs()).sum();
        assert!((sum - 1.0).abs() < 1e-12);

        // All zeros case
        let mut zeros = [0.0, 0.0, 0.0];
        apply_normalization(&mut zeros, WindowNormalization::Sum);
        assert_slice_eq(&zeros, &[0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn test_apply_normalization_rms() {
        // Normal case
        let mut slice = [1.0, 2.0, 3.0, 4.0];
        // rms = sqrt((1 + 4 + 9 + 16) / 4) = sqrt(30 / 4) = sqrt(7.5) = 2.7386127875258306
        apply_normalization(&mut slice, WindowNormalization::Rms);

        // Let's verify the RMS of the new slice is 1.0
        let sq_sum: f64 = slice.iter().map(|&v| v * v).sum();
        let rms = (sq_sum / slice.len() as f64).sqrt();
        assert!((rms - 1.0).abs() < 1e-12);

        // All zeros case
        let mut zeros = [0.0, 0.0, 0.0];
        apply_normalization(&mut zeros, WindowNormalization::Rms);
        assert_slice_eq(&zeros, &[0.0, 0.0, 0.0], 1e-12);
    }
}
