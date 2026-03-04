use std::sync::Arc;
use particelle_curve::CompiledCurve;
use crate::context::SignalContext;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    pub fn new(val: f64) -> Self {
        Self(AtomicU64::new(val.to_bits()))
    }
    pub fn get(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }
    pub fn set(&self, val: f64) {
        self.0.store(val.to_bits(), Ordering::Relaxed);
    }
}
impl Clone for AtomicF64 {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

#[derive(Debug)]
pub struct ChaosState {
    pub x: AtomicF64,
    pub y: AtomicF64,
    pub z: AtomicF64,
    pub last_frame: AtomicU64,
}

impl ChaosState {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: AtomicF64::new(x),
            y: AtomicF64::new(y),
            z: AtomicF64::new(z),
            last_frame: AtomicU64::new(u64::MAX),
        }
    }
    
    pub fn advance(&self, frame: u64) -> bool {
        let last = self.last_frame.load(Ordering::Relaxed);
        if last != frame {
            self.last_frame.store(frame, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum OscShape {
    Sine,
    Triangle,
    Saw,
    Square,
    Phasor,
}

/// The universal parameter signal type.
///
/// All engine parameters are expressed as `ParamSignal`. No raw `f64` parameters
/// exist in the public API. Every leaf value flows through this enum.
///
/// Signal graphs are built from YAML at load time and evaluated per-block.
/// Evaluation is a pure recursive descent with no allocation.
#[derive(Debug, Clone)]
pub enum ParamSignal {
    /// A constant scalar value.
    Const(f64),

    /// A compiled JSON curve evaluated at the current time (seconds).
    Curve(Arc<CompiledCurve>),

    /// A named field from the runtime control context (MIDI, MPE, etc.).
    Control { field: String },

    /// Sum of two signals.
    Sum(Box<ParamSignal>, Box<ParamSignal>),

    /// Product of two signals.
    Mul(Box<ParamSignal>, Box<ParamSignal>),

    /// Apply a named transformation function to an input signal.
    Map { input: Box<ParamSignal>, func: MapFunc },

    /// Clamp a signal to [min, max].
    Clamp { input: Box<ParamSignal>, min: f64, max: f64 },

    /// Scale and shift: `output = input * scale + offset`.
    ScaleOffset { input: Box<ParamSignal>, scale: f64, offset: f64 },

    /// Programmatic Low Frequency Oscillator (LFO). Outputs between [0.0, 1.0].
    Oscillator { shape: OscShape, freq: Box<ParamSignal>, phase: f64 },

    /// Offline audio feature analysis vector mapping (e.g. F0, RMS).
    /// Linearly interpolated at the audio `hop_rate` resolution.
    Analysis { buffer: Arc<Vec<f64>>, hop_rate: f64 },

    /// Lorenz chaotic attractor. Output dimension is "x", "y", or "z".
    Lorenz { state: Arc<ChaosState>, sigma: f64, rho: f64, beta: f64, dt: f64, out_dim: String },
    
    /// Rössler chaotic attractor. Output dimension is "x", "y", or "z".
    Rossler { state: Arc<ChaosState>, a: f64, b: f64, c: f64, dt: f64, out_dim: String },
    
    /// Hénon discrete map. Output dimension is "x" or "y".
    Henon { state: Arc<ChaosState>, a: f64, b: f64, out_dim: String },
    
    /// Brownian motion (Random Walk) mapping to output dimension "x".
    Brownian { state: Arc<ChaosState>, sigma: f64, dt: f64 },
}

impl ParamSignal {
    /// Evaluate the signal graph at the current `ctx`.
    ///
    /// This function must not allocate. All branching is statically dispatched.
    pub fn eval(&self, ctx: &SignalContext<'_>) -> f64 {
        match self {
            ParamSignal::Const(v) => *v,
            ParamSignal::Curve(curve) => {
                let t = ctx.frame as f64 / ctx.sample_rate;
                curve.eval(t)
            }
            ParamSignal::Control { field } => {
                ctx.fields.get(field).unwrap_or(0.0)
            }
            ParamSignal::Sum(a, b) => a.eval(ctx) + b.eval(ctx),
            ParamSignal::Mul(a, b) => a.eval(ctx) * b.eval(ctx),
            ParamSignal::Map { input, func } => func.apply(input.eval(ctx), ctx),
            ParamSignal::Clamp { input, min, max } => {
                input.eval(ctx).clamp(*min, *max)
            }
            ParamSignal::ScaleOffset { input, scale, offset } => {
                input.eval(ctx) * scale + offset
            }
            ParamSignal::Oscillator { shape, freq, phase } => {
                let t = ctx.frame as f64 / ctx.sample_rate;
                let hz = freq.eval(ctx);
                let current_phase = (t * hz + *phase).fract(); // [0.0, 1.0)
                
                match shape {
                    OscShape::Phasor => current_phase,
                    OscShape::Saw => current_phase, // Same as phasor natively
                    OscShape::Square => if current_phase < 0.5 { 1.0 } else { 0.0 },
                    OscShape::Triangle => {
                        let mut v = current_phase * 4.0;
                        if v > 2.0 { v = 4.0 - v; }
                        v * 0.5 // Map from [0, 2] down to [0, 1]
                    }
                    OscShape::Sine => {
                        let phase_rad = current_phase * std::f64::consts::TAU;
                        let sine_val = phase_rad.sin(); // [-1.0, 1.0]
                        (sine_val * 0.5) + 0.5 // Remap [-1, 1] to [0, 1]
                    }
                }
            }
            ParamSignal::Analysis { buffer, hop_rate } => {
                if buffer.is_empty() {
                    return 0.0;
                }
                
                let time_sec = ctx.frame as f64 / ctx.sample_rate;
                let exact_idx = time_sec * hop_rate;
                
                let idx_floor = exact_idx.floor() as usize;
                let idx_ceil = idx_floor + 1;
                
                if idx_ceil >= buffer.len() {
                    return *buffer.last().unwrap_or(&0.0);
                }
                
                let frac = exact_idx - exact_idx.floor();
                let val_a = buffer[idx_floor];
                let val_b = buffer[idx_ceil];
                
                val_a + (val_b - val_a) * frac
            }
            ParamSignal::Lorenz { state, sigma, rho, beta, dt, out_dim } => {
                let frame = ctx.frame;
                if state.advance(frame) {
                    let mut x = state.x.get();
                    let mut y = state.y.get();
                    let mut z = state.z.get();
                    
                    let dx = sigma * (y - x);
                    let dy = x * (rho - z) - y;
                    let dz = x * y - beta * z;
                    
                    x += dx * dt;
                    y += dy * dt;
                    z += dz * dt;
                    
                    state.x.set(x);
                    state.y.set(y);
                    state.z.set(z);
                }
                match out_dim.as_str() {
                    "y" => state.y.get(),
                    "z" => state.z.get(),
                    _ => state.x.get(),
                }
            }
            ParamSignal::Rossler { state, a, b, c, dt, out_dim } => {
                let frame = ctx.frame;
                if state.advance(frame) {
                    let mut x = state.x.get();
                    let mut y = state.y.get();
                    let mut z = state.z.get();
                    
                    let dx = -y - z;
                    let dy = x + a * y;
                    let dz = b + z * (x - c);
                    
                    x += dx * dt;
                    y += dy * dt;
                    z += dz * dt;
                    
                    state.x.set(x);
                    state.y.set(y);
                    state.z.set(z);
                }
                match out_dim.as_str() {
                    "y" => state.y.get(),
                    "z" => state.z.get(),
                    _ => state.x.get(),
                }
            }
            ParamSignal::Henon { state, a, b, out_dim } => {
                let frame = ctx.frame;
                if state.advance(frame) {
                    let x = state.x.get();
                    let y = state.y.get();
                    
                    let nx = 1.0 - a * x * x + y;
                    let ny = b * x;
                    
                    state.x.set(nx);
                    state.y.set(ny);
                }
                match out_dim.as_str() {
                    "y" => state.y.get(),
                    _ => state.x.get(),
                }
            }
            ParamSignal::Brownian { state, sigma, dt } => {
                let frame = ctx.frame;
                if state.advance(frame) {
                    let val = state.x.get();
                    // Fast pseudo-random via seeded LCG
                    let mut seed = frame.wrapping_mul(6364136223846793005).wrapping_add(state.last_frame.load(Ordering::Relaxed));
                    seed ^= seed >> 32;
                    let noise = (seed as f64 / u64::MAX as f64) * 2.0 - 1.0; // Uniform [-1, 1] Approximation
                    
                    let next = val + noise * sigma * dt;
                    state.x.set(next);
                }
                state.x.get()
            }
        }
    }
}

/// Named value transformation functions applied as `Map` nodes.
#[derive(Debug, Clone)]
pub enum MapFunc {
    DbToLinear,
    LinearToDb,
    /// MIDI note number → Hz (equal temperament; tuning overrides in engine).
    MidiNoteToHz,
    HzToMidiNote,
    Abs,
    Negate,
    Recip,
    /// Custom named transformer; implementation resolved at engine setup.
    Custom { name: String },
}

impl MapFunc {
    pub fn apply(&self, v: f64, ctx: &SignalContext<'_>) -> f64 {
        match self {
            MapFunc::DbToLinear => 10.0f64.powf(v / 20.0),
            MapFunc::LinearToDb => 20.0 * v.abs().max(f64::MIN_POSITIVE).log10(),
            MapFunc::MidiNoteToHz => 440.0 * 2.0f64.powf((v - 69.0) / 12.0),
            MapFunc::HzToMidiNote => 69.0 + 12.0 * (v / 440.0).log2(),
            MapFunc::Abs => v.abs(),
            MapFunc::Negate => -v,
            MapFunc::Recip => if v == 0.0 { 0.0 } else { 1.0 / v },
            MapFunc::Custom { name } => {
                ctx.resolve_custom_map(name, v)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{SignalContext, NullFields};

    fn ctx() -> SignalContext<'static> {
        SignalContext { frame: 0, sample_rate: 48000.0, fields: &NullFields, custom_resolver: None }
    }

    #[test]
    fn const_signal() {
        assert_eq!(ParamSignal::Const(3.14).eval(&ctx()), 3.14);
    }

    #[test]
    fn sum_signal() {
        let s = ParamSignal::Sum(
            Box::new(ParamSignal::Const(1.0)),
            Box::new(ParamSignal::Const(2.0)),
        );
        assert_eq!(s.eval(&ctx()), 3.0);
    }

    #[test]
    fn mul_signal() {
        let s = ParamSignal::Mul(
            Box::new(ParamSignal::Const(2.0)),
            Box::new(ParamSignal::Const(3.0)),
        );
        assert_eq!(s.eval(&ctx()), 6.0);
    }

    #[test]
    fn clamp_signal() {
        let s = ParamSignal::Clamp {
            input: Box::new(ParamSignal::Const(5.0)),
            min: 0.0,
            max: 3.0,
        };
        assert_eq!(s.eval(&ctx()), 3.0);
    }

    #[test]
    fn db_to_linear_0db() {
        let v = MapFunc::DbToLinear.apply(0.0, &ctx());
        assert!((v - 1.0).abs() < 1e-14);
    }

    #[test]
    fn midi_note_69_is_440hz() {
        let v = MapFunc::MidiNoteToHz.apply(69.0, &ctx());
        assert!((v - 440.0).abs() < 1e-10);
    }
}
