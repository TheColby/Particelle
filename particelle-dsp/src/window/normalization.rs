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
