//! Fixed-topology M0 synthesis engine.

use crate::dsp::{M0VoiceDsp, WavetableBank};
use crate::mpe::{MpeVoiceManager, VoicePhase, ZoneConfig};
use crate::rt_audit::AudioThreadGuard;
use particelle_midi::{MidiEvent, MidiEventKind};
use particelle_tuning::ScalaTuning;

const CONTROL_INTERVAL_FRAMES: u64 = 16;

#[derive(Debug, Clone, Copy)]
pub struct M0Config {
    pub sample_rate: f64,
    pub max_voices: usize,
    pub base_morph: f32,
    /// Slow macro modulation applied to the wavetable/body morph.
    pub modulation_hz: f32,
    /// Wet amount for the fixed feedback delay.
    pub delay_mix: f32,
    /// Wet amount for the fixed diffuse reverb.
    pub reverb_mix: f32,
    pub zone: ZoneConfig,
}

impl Default for M0Config {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            max_voices: 16,
            base_morph: 0.5,
            modulation_hz: 0.0,
            delay_mix: 0.0,
            reverb_mix: 0.0,
            zone: ZoneConfig::lower(15),
        }
    }
}

pub struct M0Engine {
    config: M0Config,
    frame: u64,
    voices: MpeVoiceManager,
    voice_dsp: Vec<M0VoiceDsp>,
    base_frequencies: Vec<f64>,
    tables: WavetableBank,
    tuning: ScalaTuning,
    external_tuning: [Option<f64>; 128],
    effects: GlobalEffects,
}

impl M0Engine {
    pub fn new(config: M0Config, tuning: ScalaTuning) -> Result<Self, M0EngineError> {
        config.validate()?;
        let mut voice_dsp = Vec::with_capacity(config.max_voices);
        voice_dsp.resize_with(config.max_voices, M0VoiceDsp::new);
        Ok(Self {
            config,
            frame: 0,
            voices: MpeVoiceManager::new(config.zone, config.max_voices),
            voice_dsp,
            base_frequencies: vec![440.0; config.max_voices],
            tables: WavetableBank::new(),
            tuning,
            external_tuning: [None; 128],
            effects: GlobalEffects::new(config.sample_rate),
        })
    }

    /// Validate configuration on the control thread before construction.
    pub fn validate_config(config: M0Config) -> Result<(), M0EngineError> {
        config.validate()
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.active_voice_count()
    }

    pub fn voices(&self) -> &[crate::mpe::VoiceState] {
        self.voices.voices()
    }

    /// Replace the tuning of all MIDI notes from a host tuning service.
    ///
    /// This is the MTS-ESP bridge point: a VST3/AU adapter obtains its current
    /// 128-note table and submits it at a block boundary. Invalid values are
    /// rejected rather than reaching the audio path.
    pub fn set_external_tuning_table(
        &mut self,
        frequencies_hz: [f64; 128],
    ) -> Result<(), M0EngineError> {
        for frequency in frequencies_hz {
            if !frequency.is_finite() || frequency <= 0.0 {
                return Err(M0EngineError::InvalidExternalTuningFrequency(frequency));
            }
        }
        self.external_tuning = frequencies_hz.map(Some);
        Ok(())
    }

    /// Resume Scala/KBM tuning after an external host tuning table is removed.
    pub fn clear_external_tuning(&mut self) {
        self.external_tuning = [None; 128];
    }

    /// Process one block. Events must be sorted by `frame_offset`.
    ///
    /// The method allocates no memory and marks its complete scope for the
    /// optional global allocation auditor.
    pub fn process_block(&mut self, events: &[MidiEvent], output: &mut [[f32; 2]]) {
        let _audio_thread = AudioThreadGuard::enter();
        let mut event_index = 0_usize;

        for (frame_offset, frame_output) in output.iter_mut().enumerate() {
            *frame_output = self.process_frame(events, &mut event_index, frame_offset);
        }

        self.frame += output.len() as u64;
    }

    /// Render a diffuse, power-normalized bus to any channel count.
    ///
    /// One channel receives mono, two channels preserve the stereo render, and
    /// larger layouts receive the normalized mono field in every channel. This
    /// makes the output useful for arbitrary buses without claiming speaker
    /// position semantics that an M0 host adapter has not negotiated.
    pub fn process_block_diffuse(
        &mut self,
        events: &[MidiEvent],
        output: &mut [&mut [f32]],
    ) -> Result<(), M0EngineError> {
        let channels = output.len();
        if channels == 0 {
            return Err(M0EngineError::InvalidOutputLayout);
        }
        let frames = output[0].len();
        if output.iter().any(|channel| channel.len() != frames) {
            return Err(M0EngineError::InvalidOutputLayout);
        }
        let _audio_thread = AudioThreadGuard::enter();
        let mut event_index = 0_usize;
        let scale = 1.0 / (channels as f32).sqrt();
        for frame_offset in 0..frames {
            let stereo = self.process_frame(events, &mut event_index, frame_offset);
            match channels {
                1 => {
                    output[0][frame_offset] =
                        (stereo[0] + stereo[1]) * core::f32::consts::FRAC_1_SQRT_2
                }
                2 => {
                    output[0][frame_offset] = stereo[0];
                    output[1][frame_offset] = stereo[1];
                }
                _ => {
                    let diffuse =
                        (stereo[0] + stereo[1]) * core::f32::consts::FRAC_1_SQRT_2 * scale;
                    for channel in output.iter_mut() {
                        channel[frame_offset] = diffuse;
                    }
                }
            }
        }
        self.frame += frames as u64;
        Ok(())
    }

    pub fn all_notes_off(&mut self) {
        self.voices.all_notes_off(self.frame);
    }

    /// Dispatch a complete short MIDI message at the next process boundary.
    /// Hosts must call this on the same thread as `process_block` or serialize
    /// access externally; the engine itself intentionally owns no locks.
    pub fn handle_midi_bytes(&mut self, bytes: &[u8]) {
        if let Some(event) = particelle_midi::parse_midi_bytes(bytes, 0) {
            self.handle_event(&event, self.frame);
        }
    }

    fn handle_event(&mut self, event: &MidiEvent, absolute_frame: u64) {
        if let MidiEventKind::Note(note) = &event.kind {
            if note.is_on && self.frequency_for_note(note.note).is_none() {
                return;
            }
        }

        let update = self.voices.handle_event(event, absolute_frame);
        if let Some(slot) = update.started_slot {
            let voice = self.voices.voices()[slot];
            let Some(base_frequency) = self.frequency_for_note(voice.note) else {
                self.voices.free_slot(slot);
                return;
            };
            self.base_frequencies[slot] = base_frequency;
            self.voice_dsp[slot].start(&voice, base_frequency, self.config.sample_rate);
        }
    }

    fn frequency_for_note(&self, note: u8) -> Option<f64> {
        self.external_tuning[usize::from(note)]
            .or_else(|| self.tuning.frequency_for_midi_note(note))
    }

    fn process_frame(
        &mut self,
        events: &[MidiEvent],
        event_index: &mut usize,
        frame_offset: usize,
    ) -> [f32; 2] {
        while *event_index < events.len() && events[*event_index].frame_offset == frame_offset {
            self.handle_event(&events[*event_index], self.frame + frame_offset as u64);
            *event_index += 1;
        }
        let absolute_frame = self.frame + frame_offset as u64;
        let update_control = absolute_frame & (CONTROL_INTERVAL_FRAMES - 1) == 0;
        let morph = (self.config.base_morph
            + (absolute_frame as f64 * self.config.modulation_hz as f64 * core::f64::consts::TAU
                / self.config.sample_rate)
                .sin() as f32
                * 0.15)
            .clamp(0.0, 1.0);
        let mut output = [0.0, 0.0];
        for slot in 0..self.voice_dsp.len() {
            let voice = self.voices.voices()[slot];
            if voice.phase == VoicePhase::Free {
                continue;
            }
            let stereo = self.voice_dsp[slot].process(
                &voice,
                &self.tables,
                self.base_frequencies[slot],
                morph,
                self.config.sample_rate,
                update_control,
            );
            output[0] += stereo[0];
            output[1] += stereo[1];
            if self.voice_dsp[slot].is_finished() {
                self.voices.free_slot(slot);
            }
        }
        self.effects
            .process(output, self.config.delay_mix, self.config.reverb_mix)
    }
}

impl M0Config {
    pub fn validate(&self) -> Result<(), M0EngineError> {
        if !self.sample_rate.is_finite() || self.sample_rate <= 0.0 {
            return Err(M0EngineError::InvalidSampleRate(self.sample_rate));
        }
        if self.max_voices == 0 {
            return Err(M0EngineError::InvalidVoiceCount);
        }
        for value in [
            self.base_morph,
            self.modulation_hz,
            self.delay_mix,
            self.reverb_mix,
        ] {
            if !value.is_finite() {
                return Err(M0EngineError::NonFiniteParameter);
            }
        }
        if !(0.0..=1.0).contains(&self.base_morph)
            || self.modulation_hz < 0.0
            || !(0.0..=1.0).contains(&self.delay_mix)
            || !(0.0..=1.0).contains(&self.reverb_mix)
        {
            return Err(M0EngineError::ParameterOutOfRange);
        }
        Ok(())
    }
}

/// Fixed-size global effects. Allocation happens only at engine construction.
struct GlobalEffects {
    delay: Vec<[f32; 2]>,
    reverb: Vec<[f32; 2]>,
    cursor: usize,
}

impl GlobalEffects {
    fn new(sample_rate: f64) -> Self {
        let delay_len = (sample_rate * 0.29).round().max(1.0) as usize;
        let reverb_len = (sample_rate * 0.071).round().max(1.0) as usize;
        Self {
            delay: vec![[0.0; 2]; delay_len],
            reverb: vec![[0.0; 2]; reverb_len],
            cursor: 0,
        }
    }

    fn process(&mut self, input: [f32; 2], delay_mix: f32, reverb_mix: f32) -> [f32; 2] {
        let delay_index = self.cursor % self.delay.len();
        let reverb_index = self.cursor % self.reverb.len();
        let delayed = self.delay[delay_index];
        let verb = self.reverb[reverb_index];
        self.delay[delay_index] = [input[0] + delayed[0] * 0.46, input[1] + delayed[1] * 0.46];
        let mono = (input[0] + input[1]) * core::f32::consts::FRAC_1_SQRT_2;
        self.reverb[reverb_index] = [mono + verb[0] * 0.67, mono + verb[1] * 0.61];
        self.cursor = self.cursor.wrapping_add(1);
        [
            ((input[0] + delayed[0] * delay_mix + verb[0] * reverb_mix) * 0.72).tanh(),
            ((input[1] + delayed[1] * delay_mix + verb[1] * reverb_mix) * 0.72).tanh(),
        ]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum M0EngineError {
    #[error("invalid sample rate {0}")]
    InvalidSampleRate(f64),
    #[error("max voice count must be greater than zero")]
    InvalidVoiceCount,
    #[error("M0 parameters must be finite")]
    NonFiniteParameter,
    #[error("M0 parameter is outside its supported range")]
    ParameterOutOfRange,
    #[error("external tuning frequency must be finite and positive, got {0}")]
    InvalidExternalTuningFrequency(f64),
    #[error("output must contain at least one equally sized channel")]
    InvalidOutputLayout,
}

pub fn default_twelve_tet_tuning() -> ScalaTuning {
    const SCL: &str = "\
12-tone equal temperament
12
100.0
200.0
300.0
400.0
500.0
600.0
700.0
800.0
900.0
1000.0
1100.0
1200.0
";
    const KBM: &str = "\
12
0
127
60
69
440.0
12
0
1
2
3
4
5
6
7
8
9
10
11
";
    ScalaTuning::from_text(SCL, Some(KBM), 440.0).expect("the built-in 12-TET tuning must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rt_audit::{audio_allocation_count, reset_audio_allocation_count};
    use particelle_midi::NoteEvent;

    fn note(frame_offset: usize, channel: u8, note: u8, velocity: f64, is_on: bool) -> MidiEvent {
        MidiEvent {
            frame_offset,
            kind: MidiEventKind::Note(NoteEvent {
                channel,
                note,
                velocity,
                is_on,
            }),
        }
    }

    #[test]
    fn engine_renders_a_nonzero_mpe_voice() {
        let mut engine = M0Engine::new(M0Config::default(), default_twelve_tet_tuning()).unwrap();
        let mut output = vec![[0.0; 2]; 512];
        engine.process_block(&[note(0, 2, 60, 1.0, true)], &mut output);
        assert!(output
            .iter()
            .any(|frame| frame[0] != 0.0 || frame[1] != 0.0));
        assert_eq!(engine.active_voice_count(), 1);
    }

    #[test]
    fn identical_engines_render_identical_samples() {
        let config = M0Config::default();
        let mut first = M0Engine::new(config, default_twelve_tet_tuning()).unwrap();
        let mut second = M0Engine::new(config, default_twelve_tet_tuning()).unwrap();
        let events = [note(0, 2, 60, 0.8, true), note(300, 2, 60, 0.5, false)];
        let mut a = vec![[0.0; 2]; 512];
        let mut b = vec![[0.0; 2]; 512];
        first.process_block(&events, &mut a);
        second.process_block(&events, &mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn unmapped_kbm_key_does_not_consume_a_voice() {
        let scl = "\
two-tone test
2
3/2
2/1
";
        let kbm = "\
2
60
61
60
60
261.625565
2
0
x
";
        let tuning = ScalaTuning::from_text(scl, Some(kbm), 440.0).unwrap();
        let mut engine = M0Engine::new(M0Config::default(), tuning).unwrap();
        let mut output = vec![[0.0; 2]; 32];
        engine.process_block(&[note(0, 2, 61, 1.0, true)], &mut output);
        assert_eq!(engine.active_voice_count(), 0);
        assert!(output.iter().all(|frame| *frame == [0.0, 0.0]));
    }

    #[test]
    fn process_block_performs_no_audio_thread_allocations() {
        let mut engine = M0Engine::new(M0Config::default(), default_twelve_tet_tuning()).unwrap();
        let events = [note(0, 2, 60, 0.8, true)];
        let mut output = vec![[0.0; 2]; 128];
        // Warm up thread-local state and math-library paths before measuring.
        engine.process_block(&[], &mut output);
        reset_audio_allocation_count();
        engine.process_block(&events, &mut output);
        assert_eq!(audio_allocation_count(), 0);
    }

    #[test]
    fn diffuse_renderer_writes_every_channel_without_allocations() {
        let mut engine = M0Engine::new(M0Config::default(), default_twelve_tet_tuning()).unwrap();
        let events = [note(0, 2, 60, 0.8, true)];
        let mut a = [0.0; 128];
        let mut b = [0.0; 128];
        let mut c = [0.0; 128];
        let mut d = [0.0; 128];
        let mut channels: [&mut [f32]; 4] = [&mut a, &mut b, &mut c, &mut d];
        engine
            .process_block_diffuse(&events, &mut channels)
            .unwrap();
        assert!(a.iter().any(|sample| *sample != 0.0));
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(c, d);
    }

    #[test]
    fn external_tuning_requires_positive_frequencies() {
        let mut engine = M0Engine::new(M0Config::default(), default_twelve_tet_tuning()).unwrap();
        let mut table = [440.0; 128];
        table[60] = 0.0;
        assert!(matches!(
            engine.set_external_tuning_table(table),
            Err(M0EngineError::InvalidExternalTuningFrequency(0.0))
        ));
    }
}
