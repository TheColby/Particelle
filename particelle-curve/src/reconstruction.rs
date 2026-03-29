use serde::{Deserialize, Serialize};

/// Control-rate to audio-rate reconstruction methods.
///
/// A curve produces values at control rate (e.g., every 64 or 256 samples).
/// The reconstruction method defines how those sparse values are densified
/// to the audio sample rate before entering the DSP graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum ReconstructionMethod {
    /// Zero-order hold: hold control value until the next update.
    Zoh,
    /// Linear interpolation between control points.
    Linear,
    /// Cubic polynomial interpolation.
    Cubic,
    /// Monotone cubic (Fritsch-Carlson): cubic with no overshoot.
    MonotoneCubic,
    /// Windowed sinc interpolation. `taps` must be even and > 0.
    Sinc { taps: usize },
    /// Single-pole IIR low-pass. `coefficient` in [0, 1).
    OnePole { coefficient: f64 },
    /// Two-pole IIR. `a1`, `a2` are the feedback coefficients.
    TwoPole { a1: f64, a2: f64 },
    /// Slew limiter: bounds the per-sample rate of change.
    SlewLimiter { max_rate: f64 },
    /// Minimum-phase BLEP step for click-free discontinuities.
    MinblepStep,
}

/// A stateful processor that densifies control-rate values to audio rate.
pub trait Reconstructor: Send + Sync {
    /// Process a new control-rate value and produce the next audio-rate sample.
    /// `new_value` is only provided when the control rate clock ticks.
    fn next(&mut self, control_value: Option<f64>) -> f64;
    /// Reset internal state.
    fn reset(&mut self);
}

/// Create a new reconstructor based on the specified method.
pub fn create_reconstructor(method: &ReconstructionMethod) -> Box<dyn Reconstructor> {
    match method {
        ReconstructionMethod::Zoh => Box::new(ZohReconstructor::default()),
        ReconstructionMethod::Linear => Box::new(LinearReconstructor::default()),
        ReconstructionMethod::OnePole { coefficient } => {
            Box::new(OnePoleReconstructor::new(*coefficient))
        }
        ReconstructionMethod::SlewLimiter { max_rate } => {
            Box::new(SlewReconstructor::new(*max_rate))
        }
        // fallback to Linear for more complex variants not yet fully implemented
        _ => Box::new(LinearReconstructor::default()),
    }
}

#[derive(Default)]
struct ZohReconstructor {
    value: f64,
}

impl Reconstructor for ZohReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(v) = control_value {
            self.value = v;
        }
        self.value
    }
    fn reset(&mut self) {
        self.value = 0.0;
    }
}

#[derive(Default)]
struct LinearReconstructor {
    current: f64,
    target: f64,
    step: f64,
    steps_left: usize,
}

impl Reconstructor for LinearReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(v) = control_value {
            self.target = v;
            // Note: In a real engine, we'd need to know the block size / control rate
            // mapping here or receive it. For phase 1, we assume a standard step.
            // This is a simplification.
            self.step = (self.target - self.current) / 64.0; // Assume 64 sample control rate
            self.steps_left = 64;
        }

        if self.steps_left > 0 {
            self.current += self.step;
            self.steps_left -= 1;
            if self.steps_left == 0 {
                self.current = self.target;
            }
        }
        self.current
    }
    fn reset(&mut self) {
        self.current = 0.0;
        self.target = 0.0;
        self.step = 0.0;
        self.steps_left = 0;
    }
}

struct OnePoleReconstructor {
    coeff: f64,
    state: f64,
}

impl OnePoleReconstructor {
    fn new(coeff: f64) -> Self {
        Self {
            coeff: coeff.clamp(0.0, 0.9999),
            state: 0.0,
        }
    }
}

impl Reconstructor for OnePoleReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(v) = control_value {
            // Internal logic: state = v * (1-c) + state * c
            // But we typically evaluate this per audio sample.
            self.state = v * (1.0 - self.coeff) + self.state * self.coeff;
        } else {
            // If no new value, we just keep filtering towards the last target
            // This is slightly ambiguous for a reconstructor - usually we upsample first.
            // Placeholder for now.
        }
        self.state
    }
    fn reset(&mut self) {
        self.state = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoh_reconstructor() {
        let mut recon = ZohReconstructor::default();

        // Initial state
        assert_eq!(recon.next(None), 0.0);

        // Update value
        assert_eq!(recon.next(Some(1.0)), 1.0);

        // Hold value
        assert_eq!(recon.next(None), 1.0);
        assert_eq!(recon.next(None), 1.0);

        // Reset
        recon.reset();
        assert_eq!(recon.next(None), 0.0);
    }

    #[test]
    fn test_linear_reconstructor() {
        let mut recon = LinearReconstructor::default();

        // Step size will be 1.0 / 64.0 = 0.015625
        let first_val = recon.next(Some(1.0));
        assert_eq!(first_val, 1.0 / 64.0);

        // Move forward a few steps
        assert_eq!(recon.next(None), 2.0 / 64.0);
        assert_eq!(recon.next(None), 3.0 / 64.0);

        // Jump to end of 64 steps
        for _ in 0..60 {
            recon.next(None);
        }

        // Last step should reach exactly target (1.0)
        let last_val = recon.next(None);
        assert!((last_val - 1.0).abs() < f64::EPSILON);

        // After reaching target, it holds the value
        assert_eq!(recon.next(None), 1.0);

        // Reset
        recon.reset();
        assert_eq!(recon.next(None), 0.0);
    }

    #[test]
    fn test_one_pole_reconstructor() {
        // coeff = 0.5, formula: new_state = v * 0.5 + old_state * 0.5
        let mut recon = OnePoleReconstructor::new(0.5);

        // First step towards 1.0 from 0.0
        let val1 = recon.next(Some(1.0));
        assert_eq!(val1, 0.5);

        // No new control value keeps it at current state (current implementation behavior)
        let val_held = recon.next(None);
        assert_eq!(val_held, 0.5);

        // Next step towards 1.0
        let val2 = recon.next(Some(1.0));
        assert_eq!(val2, 0.75);

        recon.reset();
        assert_eq!(recon.next(None), 0.0);
    }

    #[test]
    fn test_slew_reconstructor() {
        // Max rate = 0.1 per sample update
        let mut recon = SlewReconstructor::new(0.1);

        // Try to jump to 1.0, should be limited to 0.1
        let val1 = recon.next(Some(1.0));
        assert_eq!(val1, 0.1);

        // No new control value holds current state
        let val_held = recon.next(None);
        assert_eq!(val_held, 0.1);

        // Try to jump to -1.0, should be limited to 0.1 - 0.1 = 0.0
        let val2 = recon.next(Some(-1.0));
        assert!(val2.abs() < f64::EPSILON);

        recon.reset();
        assert_eq!(recon.next(None), 0.0);
    }

    #[test]
    fn test_create_reconstructor() {
        // Test Zoh
        let mut zoh = create_reconstructor(&ReconstructionMethod::Zoh);
        assert_eq!(zoh.next(Some(5.0)), 5.0);
        assert_eq!(zoh.next(None), 5.0);

        // Test Linear
        let mut linear = create_reconstructor(&ReconstructionMethod::Linear);
        assert_eq!(linear.next(Some(64.0)), 1.0);

        // Test OnePole
        let mut onepole = create_reconstructor(&ReconstructionMethod::OnePole { coefficient: 0.9 });
        let val = onepole.next(Some(1.0));
        assert!((val - 0.1).abs() < 1e-6); // 1.0*(1-0.9) + 0*0.9 = 0.1

        // Test SlewLimiter
        let mut slew = create_reconstructor(&ReconstructionMethod::SlewLimiter { max_rate: 0.5 });
        assert_eq!(slew.next(Some(10.0)), 0.5);

        // Test Fallback (Cubic, etc.) -> should fall back to Linear
        let mut fallback = create_reconstructor(&ReconstructionMethod::Cubic);
        assert_eq!(fallback.next(Some(64.0)), 1.0);
    }
}

struct SlewReconstructor {
    max_rate: f64,
    state: f64,
}

impl SlewReconstructor {
    fn new(max_rate: f64) -> Self {
        Self {
            max_rate,
            state: 0.0,
        }
    }
}

impl Reconstructor for SlewReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(v) = control_value {
            let diff = v - self.state;
            self.state += diff.clamp(-self.max_rate, self.max_rate);
        }
        self.state
    }
    fn reset(&mut self) {
        self.state = 0.0;
    }
}
