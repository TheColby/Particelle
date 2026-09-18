//! Fixed M0 signal chain: wavetable excitation, coupled modal body, TPT SVF.

use crate::mpe::{VoicePhase, VoiceState};

pub const WAVETABLE_SIZE: usize = 2048;
pub const MODE_COUNT: usize = 16;
const BAND_LIMITS: [usize; 6] = [1, 2, 4, 8, 16, 32];

const MODE_RATIOS: [f64; MODE_COUNT] = [
    1.0, 2.01, 2.98, 4.07, 5.13, 6.21, 7.34, 8.57, 9.94, 11.47, 13.18, 15.08, 17.21, 19.62, 22.31,
    25.37,
];

pub struct WavetableBank {
    sine: [f32; WAVETABLE_SIZE],
    bright_bands: [[f32; WAVETABLE_SIZE]; BAND_LIMITS.len()],
}

impl WavetableBank {
    pub fn new() -> Self {
        let mut sine = [0.0; WAVETABLE_SIZE];
        let mut bright_bands = [[0.0; WAVETABLE_SIZE]; BAND_LIMITS.len()];
        for index in 0..WAVETABLE_SIZE {
            let phase = index as f64 / WAVETABLE_SIZE as f64;
            sine[index] = (std::f64::consts::TAU * phase).sin() as f32;

            for (band, maximum_harmonic) in BAND_LIMITS.iter().enumerate() {
                let mut sample = 0.0_f64;
                for harmonic in 1..=*maximum_harmonic {
                    sample +=
                        (std::f64::consts::TAU * phase * harmonic as f64).sin() / harmonic as f64;
                }
                bright_bands[band][index] = (sample * 0.52) as f32;
            }
        }
        Self { sine, bright_bands }
    }

    /// Sample a band-limited bright table. Harmonics above 45% of Nyquist are
    /// excluded before interpolation so high notes do not fold back as aliasing.
    pub fn sample(&self, phase: f64, brightness: f32, normalized_frequency: f64) -> f32 {
        let position = phase.fract() * WAVETABLE_SIZE as f64;
        let index = position.floor() as usize % WAVETABLE_SIZE;
        let next = (index + 1) % WAVETABLE_SIZE;
        let fraction = (position - position.floor()) as f32;
        let sine = self.sine[index] + (self.sine[next] - self.sine[index]) * fraction;
        let allowed_harmonics = (0.45 / normalized_frequency.max(1.0e-9)).floor() as usize;
        let band = BAND_LIMITS
            .iter()
            .rposition(|limit| *limit <= allowed_harmonics)
            .unwrap_or(0);
        let table = &self.bright_bands[band];
        let bright = table[index] + (table[next] - table[index]) * fraction;
        sine + (bright - sine) * brightness.clamp(0.0, 1.0)
    }
}

impl Default for WavetableBank {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
struct ModalMode {
    y1: f32,
    y2: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    gain: f32,
    enabled: bool,
}

impl ModalMode {
    const fn new() -> Self {
        Self {
            y1: 0.0,
            y2: 0.0,
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            gain: 0.0,
            enabled: false,
        }
    }

    fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    fn configure(&mut self, frequency_hz: f64, decay_seconds: f64, gain: f32, sample_rate: f64) {
        if frequency_hz <= 0.0 || frequency_hz >= sample_rate * 0.475 {
            self.enabled = false;
            return;
        }
        let radius = (-1.0 / (decay_seconds.max(0.015) * sample_rate)).exp();
        let omega = std::f64::consts::TAU * frequency_hz / sample_rate;
        self.a1 = (2.0 * radius * omega.cos()) as f32;
        self.a2 = (-(radius * radius)) as f32;
        self.b0 = (1.0 - radius) as f32;
        self.gain = gain;
        self.enabled = true;
    }

    fn process(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let output = self.b0 * input + self.a1 * self.y1 + self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = if output.is_finite() { output } else { 0.0 };
        self.y1 * self.gain
    }
}

#[derive(Clone, Copy)]
struct TptLowPass {
    ic1eq: f32,
    ic2eq: f32,
    a1: f32,
    a2: f32,
    a3: f32,
}

impl TptLowPass {
    const fn new() -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
            a1: 1.0,
            a2: 0.0,
            a3: 0.0,
        }
    }

    fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    fn configure(&mut self, cutoff_hz: f64, resonance: f64, sample_rate: f64) {
        let cutoff = cutoff_hz.clamp(20.0, sample_rate * 0.45);
        let g = (std::f64::consts::PI * cutoff / sample_rate).tan();
        let k = 2.0 - 1.85 * resonance.clamp(0.0, 1.0);
        let a1 = 1.0 / (1.0 + g * (g + k));
        self.a1 = a1 as f32;
        self.a2 = (g * a1) as f32;
        self.a3 = (g * g * a1) as f32;
    }

    fn process(&mut self, input: f32) -> f32 {
        let v3 = input - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a2 * v3;
        let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        v2
    }
}

pub struct M0VoiceDsp {
    voice_id: u64,
    oscillator_phase: f64,
    modes: [ModalMode; MODE_COUNT],
    filter: TptLowPass,
    envelope: f32,
    impulse: f32,
    smoothed_pressure: f32,
    smoothed_timbre: f32,
    smoothed_frequency: f64,
    direct_energy: f32,
    body_energy: f32,
    rng: u64,
    pan: f32,
    active: bool,
}

impl M0VoiceDsp {
    pub const fn new() -> Self {
        Self {
            voice_id: 0,
            oscillator_phase: 0.0,
            modes: [ModalMode::new(); MODE_COUNT],
            filter: TptLowPass::new(),
            envelope: 0.0,
            impulse: 0.0,
            smoothed_pressure: 0.0,
            smoothed_timbre: 0.5,
            smoothed_frequency: 440.0,
            direct_energy: 1.0e-6,
            body_energy: 1.0e-6,
            rng: 1,
            pan: 0.0,
            active: false,
        }
    }

    pub fn start(&mut self, voice: &VoiceState, base_frequency_hz: f64, sample_rate: f64) {
        self.voice_id = voice.id;
        self.oscillator_phase = 0.0;
        self.envelope = 0.0;
        self.impulse = voice.velocity as f32;
        self.smoothed_pressure = voice.expression.pressure as f32;
        self.smoothed_timbre = voice.expression.timbre as f32;
        self.smoothed_frequency = base_frequency_hz;
        self.direct_energy = 1.0e-6;
        self.body_energy = 1.0e-6;
        self.rng = voice.id ^ (u64::from(voice.note) << 32) ^ 0x9e37_79b9_7f4a_7c15;
        self.pan = ((f32::from(voice.note) - 60.0) / 36.0).clamp(-0.8, 0.8);
        self.active = true;
        self.filter.reset();
        for mode in &mut self.modes {
            mode.reset();
        }
        self.update_coefficients(
            base_frequency_hz,
            self.smoothed_pressure,
            self.smoothed_timbre,
            sample_rate,
        );
    }

    pub fn is_finished(&self) -> bool {
        !self.active || self.envelope < 1.0e-5
    }

    pub fn process(
        &mut self,
        voice: &VoiceState,
        tables: &WavetableBank,
        base_frequency_hz: f64,
        base_morph: f32,
        sample_rate: f64,
        update_control: bool,
    ) -> [f32; 2] {
        if !self.active || voice.phase == VoicePhase::Free {
            return [0.0, 0.0];
        }

        let smoothing = (1.0 - (-1.0 / (0.006 * sample_rate)).exp()) as f32;
        self.smoothed_pressure +=
            (voice.expression.pressure as f32 - self.smoothed_pressure) * smoothing;
        self.smoothed_timbre += (voice.expression.timbre as f32 - self.smoothed_timbre) * smoothing;

        let target_frequency =
            base_frequency_hz * 2.0_f64.powf(voice.expression.pitch_bend_semitones / 12.0);
        let pitch_slew = 1.0 - (-1.0 / (0.0015 * sample_rate)).exp();
        self.smoothed_frequency += (target_frequency - self.smoothed_frequency) * pitch_slew;

        if update_control {
            self.update_coefficients(
                self.smoothed_frequency,
                self.smoothed_pressure,
                self.smoothed_timbre,
                sample_rate,
            );
        }

        let phase_increment = self.smoothed_frequency / sample_rate;
        let oscillator = tables.sample(
            self.oscillator_phase,
            self.smoothed_pressure,
            phase_increment,
        );
        self.oscillator_phase = (self.oscillator_phase + phase_increment).fract();

        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        let noise = ((self.rng >> 32) as u32 as f32 / u32::MAX as f32) * 2.0 - 1.0;

        let morph = (base_morph + (self.smoothed_timbre - 0.5) * 0.85).clamp(0.0, 1.0);
        let excitation = oscillator * (0.025 + 0.22 * morph) + noise * 0.012 * morph + self.impulse;
        self.impulse = 0.0;

        let mut body = 0.0_f32;
        for mode in &mut self.modes {
            body += mode.process(excitation);
        }
        body = (body * 2.4).tanh();

        let energy_smoothing = (1.0 - (-1.0 / (0.05 * sample_rate)).exp()) as f32;
        self.direct_energy += (oscillator * oscillator - self.direct_energy) * energy_smoothing;
        self.body_energy += (body * body - self.body_energy) * energy_smoothing;
        let body_match = ((self.direct_energy + 1.0e-6) / (self.body_energy + 1.0e-6))
            .sqrt()
            .clamp(0.25, 4.0);

        let angle = morph * std::f32::consts::FRAC_PI_2;
        let direct_gain = angle.cos();
        let body_gain = angle.sin();
        let mixed = oscillator * direct_gain + body * body_match * body_gain;
        let filtered = self.filter.process(mixed);

        match voice.phase {
            VoicePhase::Held => {
                let attack = (1.0 - (-1.0 / (0.004 * sample_rate)).exp()) as f32;
                self.envelope += (1.0 - self.envelope) * attack;
            }
            VoicePhase::Released => {
                let release_seconds = 0.25 + 1.75 * (1.0 - voice.release_velocity as f32);
                self.envelope *= (-1.0 / (f64::from(release_seconds) * sample_rate)).exp() as f32;
                if self.envelope < 1.0e-5 {
                    self.envelope = 0.0;
                    self.active = false;
                }
            }
            VoicePhase::Free => self.active = false,
        }

        let amplitude =
            self.envelope * voice.velocity as f32 * (0.62 + 0.38 * self.smoothed_pressure);
        let sample = (filtered * amplitude * 0.38).tanh();
        let left_gain = ((1.0 - self.pan) * 0.5).sqrt();
        let right_gain = ((1.0 + self.pan) * 0.5).sqrt();
        [sample * left_gain, sample * right_gain]
    }

    fn update_coefficients(
        &mut self,
        fundamental_hz: f64,
        pressure: f32,
        timbre: f32,
        sample_rate: f64,
    ) {
        let decay = 0.22 + 2.6 * f64::from(timbre.clamp(0.0, 1.0));
        for (index, mode) in self.modes.iter_mut().enumerate() {
            let frequency = fundamental_hz * MODE_RATIOS[index];
            let mode_decay = decay / (1.0 + index as f64 * 0.075);
            let gain = 1.0 / (1.0 + index as f32).sqrt();
            mode.configure(frequency, mode_decay, gain, sample_rate);
        }
        let cutoff = 350.0 + 15_000.0 * f64::from(pressure.clamp(0.0, 1.0)).powf(1.6);
        self.filter
            .configure(cutoff, 0.15 + 0.35 * f64::from(timbre), sample_rate);
    }
}

impl Default for M0VoiceDsp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bright_table_uses_a_reduced_harmonic_band_at_high_frequency() {
        let tables = WavetableBank::new();
        let low = tables.sample(0.137, 1.0, 0.01);
        let high = tables.sample(0.137, 1.0, 0.30);
        assert_ne!(low, high);
        assert!(low.is_finite() && high.is_finite());
    }
}
