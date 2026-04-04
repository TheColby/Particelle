/// One-pole IIR low-pass filter (exponential moving average).
///
/// `coeff` is in [0, 1]. 0 = instant (no smoothing), values approaching 1 = heavy smoothing.
pub struct OnePole {
    coeff: f64,
    state: f64,
}

impl OnePole {
    pub fn new(coeff: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&coeff),
            "OnePole coeff must be in [0, 1]"
        );
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
        Self {
            max_rise,
            max_fall,
            state: 0.0,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_pole_coeff_from_time() {
        // time_s <= 0 or sample_rate <= 0
        assert_eq!(OnePole::coeff_from_time(0.0, 44100.0), 0.0);
        assert_eq!(OnePole::coeff_from_time(-1.0, 44100.0), 0.0);
        assert_eq!(OnePole::coeff_from_time(1.0, 0.0), 0.0);
        assert_eq!(OnePole::coeff_from_time(1.0, -44100.0), 0.0);

        // Valid values
        let coeff1 = OnePole::coeff_from_time(0.1, 44100.0);
        let expected1 = (-1.0 / (0.1 * 44100.0_f64)).exp();
        assert!((coeff1 - expected1).abs() < 1e-12);

        let coeff2 = OnePole::coeff_from_time(1.0, 48000.0);
        let expected2 = (-1.0 / (1.0 * 48000.0_f64)).exp();
        assert!((coeff2 - expected2).abs() < 1e-12);
    }

    #[test]
    fn test_one_pole_process_and_reset() {
        let mut filter = OnePole::new(0.5); // state initially 0.0

        // Output = input + coeff * (state - input)
        // input = 1.0, state = 0.0, coeff = 0.5
        // -> state = 1.0 + 0.5 * (0.0 - 1.0) = 0.5
        assert_eq!(filter.process(1.0), 0.5);

        // input = 1.0, state = 0.5, coeff = 0.5
        // -> state = 1.0 + 0.5 * (0.5 - 1.0) = 0.75
        assert_eq!(filter.process(1.0), 0.75);

        filter.reset();

        // After reset, state is 0.0
        // input = 2.0, state = 0.0, coeff = 0.5
        // -> state = 2.0 + 0.5 * (0.0 - 2.0) = 1.0
        assert_eq!(filter.process(2.0), 1.0);
    }

    #[test]
    #[should_panic(expected = "OnePole coeff must be in [0, 1]")]
    fn test_one_pole_new_out_of_bounds() {
        OnePole::new(1.5);
    }

    #[test]
    fn test_slew_limiter() {
        let mut limiter = SlewLimiter::new(0.1, 0.2); // max_rise = 0.1, max_fall = 0.2

        // target = 1.0, state = 0.0 -> delta = 1.0
        // clamped delta = min(1.0, 0.1) = 0.1
        // new state = 0.1
        assert!((limiter.process(1.0) - 0.1).abs() < 1e-12);

        // process again with same target
        // target = 1.0, state = 0.1 -> delta = 0.9
        // clamped delta = min(0.9, 0.1) = 0.1
        // new state = 0.2
        assert!((limiter.process(1.0) - 0.2).abs() < 1e-12);

        // switch direction
        // target = -1.0, state = 0.2 -> delta = -1.2
        // clamped delta = max(-1.2, -0.2) = -0.2
        // new state = 0.0
        assert!((limiter.process(-1.0) - 0.0).abs() < 1e-12);

        // Reset
        limiter.reset();

        // target = 1.0, state = 0.0 -> delta = 1.0
        // new state = 0.1
        assert!((limiter.process(1.0) - 0.1).abs() < 1e-12);
    }
}
