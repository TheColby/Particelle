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
    #[should_panic(expected = "OnePole coeff must be in [0, 1]")]
    fn test_one_pole_new_invalid_high() {
        OnePole::new(1.1);
    }

    #[test]
    #[should_panic(expected = "OnePole coeff must be in [0, 1]")]
    fn test_one_pole_new_invalid_low() {
        OnePole::new(-0.1);
    }

    #[test]
    fn test_one_pole_coeff_from_time() {
        assert_eq!(OnePole::coeff_from_time(0.0, 44100.0), 0.0);
        assert_eq!(OnePole::coeff_from_time(-1.0, 44100.0), 0.0);
        assert_eq!(OnePole::coeff_from_time(1.0, 0.0), 0.0);
        assert_eq!(OnePole::coeff_from_time(1.0, -1.0), 0.0);

        // e^(-1 / (1 * 44100)) approx 0.999977
        let coeff = OnePole::coeff_from_time(1.0, 44100.0);
        assert!(coeff > 0.0 && coeff < 1.0);
    }

    #[test]
    fn test_one_pole_process() {
        // coeff = 0 -> instant
        let mut op = OnePole::new(0.0);
        assert_eq!(op.process(1.0), 1.0);
        assert_eq!(op.process(2.0), 2.0);

        // coeff = 0.5 -> average
        let mut op2 = OnePole::new(0.5);
        // x=1, coeff=0.5, state=0: state = 1 + 0.5*(0-1) = 0.5
        assert_eq!(op2.process(1.0), 0.5);
        // x=1, coeff=0.5, state=0.5: state = 1 + 0.5*(0.5-1) = 0.75
        assert_eq!(op2.process(1.0), 0.75);

        // reset
        op2.reset();
        assert_eq!(op2.state, 0.0);
        assert_eq!(op2.process(1.0), 0.5);
    }

    #[test]
    fn test_slew_limiter_new() {
        let sl = SlewLimiter::new(0.1, 0.2);
        assert_eq!(sl.max_rise, 0.1);
        assert_eq!(sl.max_fall, 0.2);
        assert_eq!(sl.state, 0.0);
    }

    #[test]
    fn test_slew_limiter_process_rise() {
        let mut sl = SlewLimiter::new(0.1, 0.2);

        // Target 1.0, max rise 0.1 per sample
        assert!( (sl.process(1.0) - 0.1).abs() < 1e-15 );
        assert!( (sl.process(1.0) - 0.2).abs() < 1e-15 );
        assert!( (sl.process(1.0) - 0.3).abs() < 1e-15 ); // precision issues, but correctly rising

        sl.state = 0.95; // Manually set to near target
        assert!( (sl.process(1.0) - 1.0).abs() < 1e-15 ); // Exact target reached
    }

    #[test]
    fn test_slew_limiter_process_fall() {
        let mut sl = SlewLimiter::new(0.1, 0.2);
        sl.state = 1.0;

        // Target 0.0, max fall 0.2 per sample
        assert!( (sl.process(0.0) - 0.8).abs() < 1e-15 );
        assert!( (sl.process(0.0) - 0.6).abs() < 1e-15 ); // precision issues, but correctly falling

        sl.state = 0.1; // Manually set to near target
        assert!( (sl.process(0.0) - 0.0).abs() < 1e-15 ); // Exact target reached
    }

    #[test]
    fn test_slew_limiter_reset() {
        let mut sl = SlewLimiter::new(0.1, 0.2);
        sl.process(1.0);
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
