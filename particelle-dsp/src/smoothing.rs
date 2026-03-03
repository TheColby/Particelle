/// One-pole IIR low-pass filter (exponential moving average).
///
/// `coeff` is in [0, 1]. 0 = instant (no smoothing), values approaching 1 = heavy smoothing.
pub struct OnePole {
    coeff: f64,
    state: f64,
}

impl OnePole {
    pub fn new(coeff: f64) -> Self {
        assert!((0.0..=1.0).contains(&coeff), "OnePole coeff must be in [0, 1]");
        Self { coeff, state: 0.0 }
    }

    /// Compute coefficient from a time constant in seconds at a given sample rate.
    pub fn coeff_from_time(time_s: f64, sample_rate: f64) -> f64 {
        if time_s <= 0.0 || sample_rate <= 0.0 {
            return 0.0;
        }
        (-1.0 / (time_s * sample_rate)).exp()
    }

    pub fn process(&mut self, x: f64) -> f64 {
        self.state = x + self.coeff * (self.state - x);
        self.state
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_pole_new_valid_coeffs() {
        let op = OnePole::new(0.0);
        assert_eq!(op.coeff, 0.0);
        assert_eq!(op.state, 0.0);

        let op = OnePole::new(0.5);
        assert_eq!(op.coeff, 0.5);
        assert_eq!(op.state, 0.0);

        let op = OnePole::new(1.0);
        assert_eq!(op.coeff, 1.0);
        assert_eq!(op.state, 0.0);
    }

    #[test]
    #[should_panic(expected = "OnePole coeff must be in [0, 1]")]
    fn test_one_pole_new_invalid_coeff_negative() {
        OnePole::new(-0.1);
    }

    #[test]
    #[should_panic(expected = "OnePole coeff must be in [0, 1]")]
    fn test_one_pole_new_invalid_coeff_greater_than_one() {
        OnePole::new(1.1);
    }
}

/// Slew limiter: limits the rate of change (rise and fall) of a signal.
pub struct SlewLimiter {
    /// Maximum rise per sample.
    pub max_rise: f64,
    /// Maximum fall per sample (positive value).
    pub max_fall: f64,
    state: f64,
}

impl SlewLimiter {
    pub fn new(max_rise: f64, max_fall: f64) -> Self {
        Self { max_rise, max_fall, state: 0.0 }
    }

    pub fn process(&mut self, target: f64) -> f64 {
        let delta = target - self.state;
        let clamped = delta.clamp(-self.max_fall, self.max_rise);
        self.state += clamped;
        self.state
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }
}
