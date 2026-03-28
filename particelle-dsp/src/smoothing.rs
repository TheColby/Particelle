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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_pole_new() {
        let op = OnePole::new(0.5);
        assert_eq!(op.coeff, 0.5);
        assert_eq!(op.state, 0.0);
    }

    #[test]
    #[should_panic]
    fn test_one_pole_new_invalid_coeff_high() {
        OnePole::new(1.1);
    }

    #[test]
    #[should_panic]
    fn test_one_pole_new_invalid_coeff_low() {
        OnePole::new(-0.1);
    }

    #[test]
    fn test_one_pole_coeff_from_time() {
        let coeff = OnePole::coeff_from_time(1.0, 44100.0);
        assert!(coeff > 0.0 && coeff < 1.0);

        let invalid_time = OnePole::coeff_from_time(0.0, 44100.0);
        assert_eq!(invalid_time, 0.0);

        let invalid_rate = OnePole::coeff_from_time(1.0, 0.0);
        assert_eq!(invalid_rate, 0.0);
    }

    #[test]
    fn test_one_pole_process() {
        let mut op = OnePole::new(0.5);
        let res1 = op.process(1.0);
        assert_eq!(res1, 0.5);

        let res2 = op.process(1.0);
        assert_eq!(res2, 0.75);
    }

    #[test]
    fn test_one_pole_reset() {
        let mut op = OnePole::new(0.5);
        op.process(1.0);
        assert_ne!(op.state, 0.0);
        op.reset();
        assert_eq!(op.state, 0.0);
    }

    #[test]
    fn test_slew_limiter_rise() {
        let mut sl = SlewLimiter::new(0.1, 0.2);
        let res1 = sl.process(1.0);
        assert_eq!(res1, 0.1);
        let res2 = sl.process(1.0);
        assert_eq!(res2, 0.2);
    }

    #[test]
    fn test_slew_limiter_fall() {
        let mut sl = SlewLimiter::new(0.1, 0.2);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        sl.process(1.0);
        // now sl.state = 1.0
        assert!((sl.state - 1.0).abs() < f64::EPSILON);

        let res1 = sl.process(0.0);
        assert!((res1 - 0.8).abs() < 1e-10, "res1 was {}", res1);
        let res2 = sl.process(0.0);
        assert!((res2 - 0.6).abs() < 1e-10, "res2 was {}", res2);
    }

    #[test]
    fn test_slew_limiter_instant() {
        let mut sl = SlewLimiter::new(0.5, 0.5);
        let res1 = sl.process(0.3);
        assert_eq!(res1, 0.3);

        let res2 = sl.process(0.1);
        assert_eq!(res2, 0.1);
    }

    #[test]
    fn test_slew_limiter_reset() {
        let mut sl = SlewLimiter::new(0.1, 0.2);
        sl.process(1.0);
        assert_ne!(sl.state, 0.0);
        sl.reset();
        assert_eq!(sl.state, 0.0);
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
