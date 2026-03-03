use super::schema::WindowNormalization;

/// Apply normalization scaling directly to the window slice.
pub fn apply_normalization(window_slice: &mut [f64], norm: WindowNormalization) {
    if window_slice.is_empty() || norm == WindowNormalization::None {
        return;
    }

    match norm {
        WindowNormalization::Peak => {
            let max = window_slice
                .iter()
                .fold(0.0_f64, |acc, &v| acc.max(v.abs()));
            if max > 0.0 && (max - 1.0).abs() > 1e-12 {
                let scale = 1.0 / max;
                for v in window_slice {
                    *v *= scale;
                }
            }
        }
        WindowNormalization::Sum => {
            let sum: f64 = window_slice.iter().fold(0.0_f64, |acc, &v| acc + v.abs());
            if sum > 0.0 {
                let scale = 1.0 / sum;
                for v in window_slice {
                    *v *= scale;
                }
            }
        }
        WindowNormalization::Rms => {
            let sq_sum: f64 = window_slice.iter().fold(0.0_f64, |acc, &v| acc + v * v);
            let rms = (sq_sum / window_slice.len() as f64).sqrt();
            if rms > 0.0 {
                let scale = 1.0 / rms;
                for v in window_slice {
                    *v *= scale;
                }
            }
        }
        WindowNormalization::None => {}
    }
}
