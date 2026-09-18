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
    pub zone: ZoneConfig,
}

impl Default for M0Config {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            max_voices: 16,
            base_morph: 0.5,
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
}

impl M0Engine {
    pub fn new(config: M0Config, tuning: ScalaTuning) -> Result<Self, M0EngineError> {
        if !config.sample_rate.is_finite() || config.sample_rate <= 0.0 {
            return Err(M0EngineError::InvalidSampleRate(config.sample_rate));
        }
        if config.max_voices == 0 {
            return Err(M0EngineError::InvalidVoiceCount);
        }
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
        })
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

    /// Process one block. Events must be sorted by `frame_offset`.
    ///
    /// The method allocates no memory and marks its complete scope for the
    /// optional global allocation auditor.
    pub fn process_block(&mut self, events: &[MidiEvent], output: &mut [[f32; 2]]) {
        let _audio_thread = AudioThreadGuard::enter();
        let mut event_index = 0_usize;

        for (frame_offset, frame_output) in output.iter_mut().enumerate() {
            *frame_output = [0.0, 0.0];
            while event_index < events.len() && events[event_index].frame_offset == frame_offset {
                self.handle_event(&events[event_index], self.frame + frame_offset as u64);
                event_index += 1;
            }

            let absolute_frame = self.frame + frame_offset as u64;
            // The control interval is a power of two; use a mask to preserve
            // the workspace's Rust 1.70 minimum version.
            let update_control = absolute_frame & (CONTROL_INTERVAL_FRAMES - 1) == 0;
            for slot in 0..self.voice_dsp.len() {
                let voice = self.voices.voices()[slot];
                if voice.phase == VoicePhase::Free {
                    continue;
                }
                let stereo = self.voice_dsp[slot].process(
                    &voice,
                    &self.tables,
                    self.base_frequencies[slot],
                    self.config.base_morph,
                    self.config.sample_rate,
                    update_control,
                );
                frame_output[0] += stereo[0];
                frame_output[1] += stereo[1];
                if self.voice_dsp[slot].is_finished() {
                    self.voices.free_slot(slot);
                }
            }

            frame_output[0] = (frame_output[0] * 0.72).tanh();
            frame_output[1] = (frame_output[1] * 0.72).tanh();
        }

        self.frame += output.len() as u64;
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
            if note.is_on && self.tuning.frequency_for_midi_note(note.note).is_none() {
                return;
            }
        }

        let update = self.voices.handle_event(event, absolute_frame);
        if let Some(slot) = update.started_slot {
            let voice = self.voices.voices()[slot];
            let Some(base_frequency) = self.tuning.frequency_for_midi_note(voice.note) else {
                self.voices.free_slot(slot);
                return;
            };
            self.base_frequencies[slot] = base_frequency;
            self.voice_dsp[slot].start(&voice, base_frequency, self.config.sample_rate);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum M0EngineError {
    #[error("invalid sample rate {0}")]
    InvalidSampleRate(f64),
    #[error("max voice count must be greater than zero")]
    InvalidVoiceCount,
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
}
