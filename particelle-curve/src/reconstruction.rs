use serde::{Deserialize, Serialize};

/// The interval used by [`create_reconstructor`] for backward compatibility.
pub const DEFAULT_CONTROL_INTERVAL_SAMPLES: usize = 64;
const MAX_SINC_TAPS: usize = 4_096;

/// Control-rate to audio-rate reconstruction methods.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum ReconstructionMethod {
    /// Zero-order hold: hold control value until the next update.
    Zoh,
    /// Linear interpolation between control points.
    Linear,
    /// Cubic Hermite interpolation with a carried incoming tangent.
    Cubic,
    /// Monotone cubic interpolation with no overshoot.
    MonotoneCubic,
    /// Causal windowed-sinc low-pass reconstruction.
    /// Tap counts are clamped to `2..=4096` and rounded up to an even count.
    Sinc { taps: usize },
    /// Single-pole IIR low-pass. `coefficient` is clamped to `[0, 1)`.
    OnePole { coefficient: f64 },
    /// Stable two-pole IIR. Unstable coefficients are projected into the
    /// stability triangle before processing.
    TwoPole { a1: f64, a2: f64 },
    /// Slew limiter: bounds the per-sample rate of change.
    SlewLimiter { max_rate: f64 },
    /// Causal, front-loaded band-limited step transition.
    MinblepStep,
}

/// A stateful processor that densifies control-rate values to audio rate.
pub trait Reconstructor: Send + Sync {
    /// Process a new control-rate value and produce the next audio-rate sample.
    /// `control_value` is only provided when the control-rate clock ticks.
    fn next(&mut self, control_value: Option<f64>) -> f64;
    /// Reset internal state to silence.
    fn reset(&mut self);
}

/// Create a reconstructor using the backward-compatible 64-sample interval.
pub fn create_reconstructor(method: &ReconstructionMethod) -> Box<dyn Reconstructor> {
    create_reconstructor_with_interval(method, DEFAULT_CONTROL_INTERVAL_SAMPLES)
}

/// Create a reconstructor for an explicit control-rate interval.
pub fn create_reconstructor_with_interval(
    method: &ReconstructionMethod,
    control_interval_samples: usize,
) -> Box<dyn Reconstructor> {
    let interval = control_interval_samples.max(1);
    match method {
        ReconstructionMethod::Zoh => Box::new(ZohReconstructor::default()),
        ReconstructionMethod::Linear => Box::new(LinearReconstructor::new(interval)),
        ReconstructionMethod::Cubic => Box::new(CubicReconstructor::new(interval)),
        ReconstructionMethod::MonotoneCubic => Box::new(MonotoneCubicReconstructor::new(interval)),
        ReconstructionMethod::Sinc { taps } => Box::new(SincReconstructor::new(*taps, interval)),
        ReconstructionMethod::OnePole { coefficient } => {
            Box::new(OnePoleReconstructor::new(*coefficient))
        }
        ReconstructionMethod::TwoPole { a1, a2 } => Box::new(TwoPoleReconstructor::new(*a1, *a2)),
        ReconstructionMethod::SlewLimiter { max_rate } => {
            Box::new(SlewReconstructor::new(*max_rate))
        }
        ReconstructionMethod::MinblepStep => Box::new(MinblepStepReconstructor::new()),
    }
}

#[derive(Default)]
struct ZohReconstructor {
    value: f64,
}

impl Reconstructor for ZohReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.value = value;
        }
        self.value
    }

    fn reset(&mut self) {
        self.value = 0.0;
    }
}

struct LinearReconstructor {
    current: f64,
    start: f64,
    target: f64,
    position: usize,
    interval: usize,
}

impl LinearReconstructor {
    fn new(interval: usize) -> Self {
        Self {
            current: 0.0,
            start: 0.0,
            target: 0.0,
            position: interval,
            interval,
        }
    }
}

impl Reconstructor for LinearReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.start = self.current;
            self.target = value;
            self.position = 0;
        }

        if self.position < self.interval {
            self.position += 1;
            let t = self.position as f64 / self.interval as f64;
            self.current = self.start + t * (self.target - self.start);
        }
        self.current
    }

    fn reset(&mut self) {
        self.current = 0.0;
        self.start = 0.0;
        self.target = 0.0;
        self.position = self.interval;
    }
}

struct CubicReconstructor {
    current: f64,
    start: f64,
    target: f64,
    previous_target: f64,
    start_tangent: f64,
    end_tangent: f64,
    position: usize,
    interval: usize,
}

impl CubicReconstructor {
    fn new(interval: usize) -> Self {
        Self {
            current: 0.0,
            start: 0.0,
            target: 0.0,
            previous_target: 0.0,
            start_tangent: 0.0,
            end_tangent: 0.0,
            position: interval,
            interval,
        }
    }
}

impl Reconstructor for CubicReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.start = self.current;
            self.start_tangent = 0.5 * (value - self.previous_target);
            self.end_tangent = value - self.start;
            self.previous_target = self.target;
            self.target = value;
            self.position = 0;
        }

        if self.position < self.interval {
            self.position += 1;
            let t = self.position as f64 / self.interval as f64;
            self.current = cubic_hermite(
                self.start,
                self.start_tangent,
                self.target,
                self.end_tangent,
                t,
            );
        }
        self.current
    }

    fn reset(&mut self) {
        self.current = 0.0;
        self.start = 0.0;
        self.target = 0.0;
        self.previous_target = 0.0;
        self.start_tangent = 0.0;
        self.end_tangent = 0.0;
        self.position = self.interval;
    }
}

struct MonotoneCubicReconstructor {
    current: f64,
    start: f64,
    target: f64,
    position: usize,
    interval: usize,
}

impl MonotoneCubicReconstructor {
    fn new(interval: usize) -> Self {
        Self {
            current: 0.0,
            start: 0.0,
            target: 0.0,
            position: interval,
            interval,
        }
    }
}

impl Reconstructor for MonotoneCubicReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.start = self.current;
            self.target = value;
            self.position = 0;
        }

        if self.position < self.interval {
            self.position += 1;
            let t = self.position as f64 / self.interval as f64;
            let shaped = t * t * (3.0 - 2.0 * t);
            self.current = self.start + shaped * (self.target - self.start);
        }
        self.current
    }

    fn reset(&mut self) {
        self.current = 0.0;
        self.start = 0.0;
        self.target = 0.0;
        self.position = self.interval;
    }
}

struct SincReconstructor {
    target: f64,
    kernel: Vec<f64>,
    history: Vec<f64>,
    cursor: usize,
    initialized: bool,
}

impl SincReconstructor {
    fn new(taps: usize, interval: usize) -> Self {
        let taps = normalized_tap_count(taps);
        Self {
            target: 0.0,
            kernel: windowed_sinc_kernel(taps, interval),
            history: vec![0.0; taps],
            cursor: 0,
            initialized: false,
        }
    }
}

impl Reconstructor for SincReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.target = value;
            if !self.initialized {
                self.history.fill(value);
                self.initialized = true;
                return value;
            }
        }

        self.history[self.cursor] = self.target;
        self.cursor = (self.cursor + 1) % self.history.len();

        self.kernel
            .iter()
            .enumerate()
            .map(|(i, coefficient)| {
                let history_index = (self.cursor + i) % self.history.len();
                coefficient * self.history[history_index]
            })
            .sum()
    }

    fn reset(&mut self) {
        self.target = 0.0;
        self.history.fill(0.0);
        self.cursor = 0;
        self.initialized = false;
    }
}

struct OnePoleReconstructor {
    coefficient: f64,
    target: f64,
    state: f64,
}

impl OnePoleReconstructor {
    fn new(coefficient: f64) -> Self {
        let coefficient = if coefficient.is_finite() {
            coefficient.clamp(0.0, 0.999_999)
        } else {
            0.0
        };
        Self {
            coefficient,
            target: 0.0,
            state: 0.0,
        }
    }
}

impl Reconstructor for OnePoleReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.target = value;
        }
        self.state = self.target * (1.0 - self.coefficient) + self.state * self.coefficient;
        self.state
    }

    fn reset(&mut self) {
        self.target = 0.0;
        self.state = 0.0;
    }
}

struct TwoPoleReconstructor {
    a1: f64,
    a2: f64,
    feedforward: f64,
    target: f64,
    y1: f64,
    y2: f64,
}

impl TwoPoleReconstructor {
    fn new(a1: f64, a2: f64) -> Self {
        let (a1, a2) = stable_two_pole_coefficients(a1, a2);
        Self {
            a1,
            a2,
            feedforward: 1.0 + a1 + a2,
            target: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl Reconstructor for TwoPoleReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.target = value;
        }
        let output = self.feedforward * self.target - self.a1 * self.y1 - self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    fn reset(&mut self) {
        self.target = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

struct SlewReconstructor {
    max_rate: f64,
    target: f64,
    state: f64,
}

impl SlewReconstructor {
    fn new(max_rate: f64) -> Self {
        let max_rate = if max_rate.is_finite() {
            max_rate.abs()
        } else {
            0.0
        };
        Self {
            max_rate,
            target: 0.0,
            state: 0.0,
        }
    }
}

impl Reconstructor for SlewReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.target = value;
        }
        let difference = self.target - self.state;
        self.state += difference.clamp(-self.max_rate, self.max_rate);
        self.state
    }

    fn reset(&mut self) {
        self.target = 0.0;
        self.state = 0.0;
    }
}

struct MinblepStepReconstructor {
    current: f64,
    start: f64,
    target: f64,
    position: usize,
    step_response: Vec<f64>,
}

impl MinblepStepReconstructor {
    fn new() -> Self {
        let step_response = minblep_step_response();
        let position = step_response.len();
        Self {
            current: 0.0,
            start: 0.0,
            target: 0.0,
            position,
            step_response,
        }
    }
}

impl Reconstructor for MinblepStepReconstructor {
    fn next(&mut self, control_value: Option<f64>) -> f64 {
        if let Some(value) = control_value {
            self.start = self.current;
            self.target = value;
            self.position = 0;
        }

        if self.position < self.step_response.len() {
            let transition = self.step_response[self.position];
            self.current = self.start + transition * (self.target - self.start);
            self.position += 1;
        } else {
            self.current = self.target;
        }
        self.current
    }

    fn reset(&mut self) {
        self.current = 0.0;
        self.start = 0.0;
        self.target = 0.0;
        self.position = self.step_response.len();
    }
}

fn cubic_hermite(p0: f64, m0: f64, p1: f64, m1: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

fn normalized_tap_count(taps: usize) -> usize {
    let taps = taps.clamp(2, MAX_SINC_TAPS);
    if taps & 1 == 0 {
        taps
    } else {
        taps + 1
    }
}

fn windowed_sinc_kernel(taps: usize, interval: usize) -> Vec<f64> {
    let cutoff = (0.5 / interval as f64).min(0.499);
    let center = (taps - 1) as f64 / 2.0;
    let mut kernel = Vec::with_capacity(taps);

    for i in 0..taps {
        let offset = i as f64 - center;
        let sinc = if offset.abs() < f64::EPSILON {
            2.0 * cutoff
        } else {
            (2.0 * std::f64::consts::PI * cutoff * offset).sin() / (std::f64::consts::PI * offset)
        };
        let window =
            0.54 - 0.46 * (2.0 * std::f64::consts::PI * i as f64 / (taps - 1) as f64).cos();
        kernel.push(sinc * window);
    }

    let sum: f64 = kernel.iter().sum();
    for coefficient in &mut kernel {
        *coefficient /= sum;
    }
    kernel
}

fn stable_two_pole_coefficients(a1: f64, a2: f64) -> (f64, f64) {
    if !a1.is_finite() || !a2.is_finite() {
        return (0.0, 0.0);
    }

    let a2 = a2.clamp(-0.999, 0.999);
    let a1_limit = (1.0 + a2 - 1e-6).max(0.0);
    (a1.clamp(-a1_limit, a1_limit), a2)
}

fn minblep_step_response() -> Vec<f64> {
    const PROTOTYPE_LENGTH: usize = 33;
    const TRANSFORM_LENGTH: usize = 128;
    const CUTOFF: f64 = 0.45;

    // Start with a linear-phase, Blackman-windowed low-pass prototype.
    let mut prototype = vec![0.0; TRANSFORM_LENGTH];
    let center = (PROTOTYPE_LENGTH - 1) as f64 / 2.0;
    for (i, sample) in prototype.iter_mut().take(PROTOTYPE_LENGTH).enumerate() {
        let offset = i as f64 - center;
        let sinc = if offset.abs() < f64::EPSILON {
            2.0 * CUTOFF
        } else {
            (2.0 * std::f64::consts::PI * CUTOFF * offset).sin() / (std::f64::consts::PI * offset)
        };
        let phase = 2.0 * std::f64::consts::PI * i as f64 / (PROTOTYPE_LENGTH - 1) as f64;
        let window = 0.42 - 0.5 * phase.cos() + 0.08 * (2.0 * phase).cos();
        *sample = sinc * window;
    }

    // Real-cepstrum spectral factorization moves the kernel energy toward the
    // start while preserving its magnitude response.
    let spectrum = dft_real(&prototype);
    let log_magnitude: Vec<f64> = spectrum
        .iter()
        .map(|(real, imag)| real.hypot(*imag).max(1e-12).ln())
        .collect();
    let cepstrum = inverse_dft_real_spectrum(&log_magnitude);

    let mut minimum_phase_cepstrum = vec![0.0; TRANSFORM_LENGTH];
    minimum_phase_cepstrum[0] = cepstrum[0];
    for i in 1..TRANSFORM_LENGTH / 2 {
        minimum_phase_cepstrum[i] = 2.0 * cepstrum[i];
    }
    minimum_phase_cepstrum[TRANSFORM_LENGTH / 2] = cepstrum[TRANSFORM_LENGTH / 2];

    let minimum_phase_spectrum = dft_real(&minimum_phase_cepstrum)
        .into_iter()
        .map(|(real, imag)| {
            let magnitude = real.exp();
            (magnitude * imag.cos(), magnitude * imag.sin())
        })
        .collect::<Vec<_>>();
    let impulse = inverse_dft_complex(&minimum_phase_spectrum);

    let total: f64 = impulse.iter().sum();
    let mut cumulative = 0.0;
    let mut response = Vec::with_capacity(TRANSFORM_LENGTH + 1);
    response.push(0.0);
    for sample in impulse {
        cumulative += sample;
        response.push(cumulative / total);
    }
    *response.last_mut().unwrap() = 1.0;
    response
}

fn dft_real(input: &[f64]) -> Vec<(f64, f64)> {
    let length = input.len();
    (0..length)
        .map(|frequency| {
            input
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |(real, imag), (sample, value)| {
                    let angle = 2.0 * std::f64::consts::PI * frequency as f64 * sample as f64
                        / length as f64;
                    (real + value * angle.cos(), imag - value * angle.sin())
                })
        })
        .collect()
}

fn inverse_dft_real_spectrum(spectrum: &[f64]) -> Vec<f64> {
    let length = spectrum.len();
    (0..length)
        .map(|sample| {
            spectrum
                .iter()
                .enumerate()
                .map(|(frequency, value)| {
                    let angle = 2.0 * std::f64::consts::PI * frequency as f64 * sample as f64
                        / length as f64;
                    value * angle.cos()
                })
                .sum::<f64>()
                / length as f64
        })
        .collect()
}

fn inverse_dft_complex(spectrum: &[(f64, f64)]) -> Vec<f64> {
    let length = spectrum.len();
    (0..length)
        .map(|sample| {
            spectrum
                .iter()
                .enumerate()
                .map(|(frequency, (real, imag))| {
                    let angle = 2.0 * std::f64::consts::PI * frequency as f64 * sample as f64
                        / length as f64;
                    real * angle.cos() - imag * angle.sin()
                })
                .sum::<f64>()
                / length as f64
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance(reconstructor: &mut dyn Reconstructor, samples: usize) -> f64 {
        let mut output = 0.0;
        for _ in 0..samples {
            output = reconstructor.next(None);
        }
        output
    }

    #[test]
    fn zoh_holds_latest_value() {
        let mut reconstructor = create_reconstructor(&ReconstructionMethod::Zoh);
        assert_eq!(reconstructor.next(Some(0.75)), 0.75);
        assert_eq!(reconstructor.next(None), 0.75);
    }

    #[test]
    fn linear_uses_explicit_control_interval() {
        let mut reconstructor =
            create_reconstructor_with_interval(&ReconstructionMethod::Linear, 4);
        assert_eq!(reconstructor.next(Some(1.0)), 0.25);
        assert_eq!(advance(&mut *reconstructor, 3), 1.0);
    }

    #[test]
    fn cubic_is_distinct_from_linear_and_reaches_target() {
        let mut cubic = create_reconstructor_with_interval(&ReconstructionMethod::Cubic, 4);
        let first = cubic.next(Some(1.0));
        assert_ne!(first, 0.25);
        assert!((advance(&mut *cubic, 3) - 1.0).abs() < 1e-14);
    }

    #[test]
    fn monotone_cubic_never_overshoots_transition() {
        let mut reconstructor =
            create_reconstructor_with_interval(&ReconstructionMethod::MonotoneCubic, 8);
        let mut values = Vec::new();
        values.push(reconstructor.next(Some(1.0)));
        for _ in 1..8 {
            values.push(reconstructor.next(None));
        }
        assert!(values.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(values.iter().all(|value| (0.0..=1.0).contains(value)));
        assert_eq!(values.last().copied(), Some(1.0));
    }

    #[test]
    fn sinc_smooths_a_step_and_settles() {
        let mut reconstructor =
            create_reconstructor_with_interval(&ReconstructionMethod::Sinc { taps: 7 }, 4);
        assert_eq!(reconstructor.next(Some(0.0)), 0.0);
        let first = reconstructor.next(Some(1.0));
        assert!(first > -0.1 && first < 1.0);
        assert!((advance(&mut *reconstructor, 8) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn sinc_tap_count_is_bounded_and_even() {
        assert_eq!(normalized_tap_count(0), 2);
        assert_eq!(normalized_tap_count(7), 8);
        assert_eq!(normalized_tap_count(usize::MAX), MAX_SINC_TAPS);
    }

    #[test]
    fn one_pole_continues_toward_target_between_ticks() {
        let mut reconstructor =
            create_reconstructor(&ReconstructionMethod::OnePole { coefficient: 0.5 });
        assert_eq!(reconstructor.next(Some(1.0)), 0.5);
        assert_eq!(reconstructor.next(None), 0.75);
    }

    #[test]
    fn two_pole_converges_to_target() {
        let mut reconstructor =
            create_reconstructor(&ReconstructionMethod::TwoPole { a1: -1.2, a2: 0.36 });
        reconstructor.next(Some(1.0));
        let output = advance(&mut *reconstructor, 256);
        assert!((output - 1.0).abs() < 1e-10);
    }

    #[test]
    fn unstable_two_pole_coefficients_are_stabilized() {
        let mut reconstructor =
            create_reconstructor(&ReconstructionMethod::TwoPole { a1: -4.0, a2: 2.0 });
        reconstructor.next(Some(1.0));
        let output = advance(&mut *reconstructor, 1_024);
        assert!(output.is_finite());
    }

    #[test]
    fn slew_limiter_moves_every_sample() {
        let mut reconstructor =
            create_reconstructor(&ReconstructionMethod::SlewLimiter { max_rate: 0.1 });
        assert!((reconstructor.next(Some(1.0)) - 0.1).abs() < 1e-14);
        assert!((reconstructor.next(None) - 0.2).abs() < 1e-14);
    }

    #[test]
    fn minblep_step_is_causal_and_reaches_target() {
        let mut reconstructor = create_reconstructor(&ReconstructionMethod::MinblepStep);
        assert_eq!(reconstructor.next(Some(1.0)), 0.0);
        let output = advance(&mut *reconstructor, 128);
        assert!((output - 1.0).abs() < 1e-14);
    }

    #[test]
    fn minblep_impulse_energy_is_front_loaded() {
        let response = minblep_step_response();
        let impulse: Vec<f64> = response.windows(2).map(|pair| pair[1] - pair[0]).collect();
        let midpoint = impulse.len() / 2;
        let early_energy: f64 = impulse[..midpoint].iter().map(|value| value * value).sum();
        let late_energy: f64 = impulse[midpoint..].iter().map(|value| value * value).sum();
        assert!(early_energy > late_energy * 100.0);
    }

    #[test]
    fn reset_clears_processor_state() {
        let mut reconstructor =
            create_reconstructor(&ReconstructionMethod::OnePole { coefficient: 0.5 });
        reconstructor.next(Some(1.0));
        reconstructor.reset();
        assert_eq!(reconstructor.next(None), 0.0);
    }
}
