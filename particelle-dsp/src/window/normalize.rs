use super::spec::NormalizationMode;

/// Apply normalization to a window buffer in place.
pub fn normalize(window: &mut [f64], mode: NormalizationMode) {
    match mode {
        NormalizationMode::None => {}

        NormalizationMode::Peak => {
            let peak = window.iter().cloned().fold(0.0f64, |acc, v| acc.max(v.abs()));
            if peak > 0.0 {
                window.iter_mut().for_each(|s| *s /= peak);
            }
        }

        NormalizationMode::Rms => {
            let rms = (window.iter().map(|s| s * s).sum::<f64>() / window.len() as f64).sqrt();
            if rms > 0.0 {
                window.iter_mut().for_each(|s| *s /= rms);
            }
        }

        NormalizationMode::Sum => {
            let sum: f64 = window.iter().sum();
            if sum != 0.0 {
                window.iter_mut().for_each(|s| *s /= sum);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window() -> Vec<f64> {
        vec![0.25, 0.5, 1.0, 0.5, 0.25]
    }

    #[test]
    fn peak_normalization() {
        let mut w = make_window();
        normalize(&mut w, NormalizationMode::Peak);
        let peak = w.iter().cloned().fold(0.0f64, f64::max);
        assert!((peak - 1.0).abs() < 1e-14);
    }

    #[test]
    fn rms_normalization() {
        let mut w = make_window();
        normalize(&mut w, NormalizationMode::Rms);
        let rms = (w.iter().map(|s| s * s).sum::<f64>() / w.len() as f64).sqrt();
        assert!((rms - 1.0).abs() < 1e-13);
    }

    #[test]
    fn sum_normalization() {
        let mut w = make_window();
        normalize(&mut w, NormalizationMode::Sum);
        let sum: f64 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-14);
    }

    #[test]
    fn none_normalization_is_identity() {
        let original = make_window();
        let mut w = original.clone();
        normalize(&mut w, NormalizationMode::None);
        assert_eq!(w, original);
    }
}
