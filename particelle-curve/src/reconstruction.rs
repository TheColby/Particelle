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
