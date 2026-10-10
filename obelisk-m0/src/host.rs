//! Framework-neutral block adapter for native M0 plug-in wrappers.
//!
//! VST3/AU glue owns format conversion and lifecycle; this module owns the
//! narrow contract that is identical across both: control changes occur at a
//! block boundary, MIDI is sample-offset event data, and output is planar.

use crate::{M0Config, M0Controls, M0Engine, M0EngineError};
use particelle_midi::MidiEvent;
use particelle_tuning::ScalaTuning;
use serde::{Deserialize, Serialize};

const HOST_STATE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HostState {
    schema_version: u32,
    base_morph: f32,
    modulation_hz: f32,
    delay_mix: f32,
    reverb_mix: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostParameter {
    BaseMorph,
    ModulationHz,
    DelayMix,
    ReverbMix,
}

/// Single-threaded adapter for a host instance.
pub struct M0HostAdapter {
    engine: M0Engine,
    controls: M0Controls,
    output_channels: usize,
}

impl M0HostAdapter {
    pub fn new(
        config: M0Config,
        tuning: ScalaTuning,
        output_channels: usize,
    ) -> Result<Self, HostAdapterError> {
        if output_channels == 0 {
            return Err(HostAdapterError::InvalidOutputChannels);
        }
        let controls = config.controls();
        Ok(Self {
            engine: M0Engine::new(config, tuning)?,
            controls,
            output_channels,
        })
    }

    pub const fn output_channels(&self) -> usize {
        self.output_channels
    }

    pub const fn controls(&self) -> M0Controls {
        self.controls
    }

    /// Serialize session controls on the control thread. Voices, effect tails,
    /// host bus configuration, and externally supplied tuning are transient.
    pub fn save_state(&self) -> Result<Vec<u8>, HostAdapterError> {
        Ok(serde_json::to_vec(&HostState {
            schema_version: HOST_STATE_VERSION,
            base_morph: self.controls.base_morph,
            modulation_hz: self.controls.modulation_hz,
            delay_mix: self.controls.delay_mix,
            reverb_mix: self.controls.reverb_mix,
        })?)
    }

    /// Restore controls on the control thread with exclusive instance access.
    /// Parsing or validation failures leave all active controls unchanged.
    pub fn restore_state(&mut self, bytes: &[u8]) -> Result<(), HostAdapterError> {
        let state: HostState = serde_json::from_slice(bytes)?;
        if state.schema_version != HOST_STATE_VERSION {
            return Err(HostAdapterError::UnsupportedStateVersion(
                state.schema_version,
            ));
        }
        let controls = M0Controls {
            base_morph: state.base_morph,
            modulation_hz: state.modulation_hz,
            delay_mix: state.delay_mix,
            reverb_mix: state.reverb_mix,
        };
        self.engine.set_controls(controls)?;
        self.controls = controls;
        Ok(())
    }

    /// Release held notes at a transport-stop block boundary. Release envelopes
    /// and effect tails continue rendering; this does not reset the instance.
    pub fn all_notes_off(&mut self) {
        self.engine.all_notes_off();
    }

    /// Set an automatable parameter. Call before `process_planar` for the
    /// current block; no synchronization is performed internally.
    pub fn set_parameter(
        &mut self,
        parameter: HostParameter,
        value: f32,
    ) -> Result<(), HostAdapterError> {
        let mut controls = self.controls;
        match parameter {
            HostParameter::BaseMorph => controls.base_morph = value,
            HostParameter::ModulationHz => controls.modulation_hz = value,
            HostParameter::DelayMix => controls.delay_mix = value,
            HostParameter::ReverbMix => controls.reverb_mix = value,
        }
        controls.validate()?;
        self.controls = controls;
        Ok(())
    }

    pub fn set_tuning_table(&mut self, frequencies_hz: [f64; 128]) -> Result<(), HostAdapterError> {
        self.engine.set_external_tuning_table(frequencies_hz)?;
        Ok(())
    }

    pub fn clear_tuning_table(&mut self) {
        self.engine.clear_external_tuning();
    }

    /// Render a planar host bus. Every slice must have the same frame count.
    pub fn process_planar(
        &mut self,
        events: &[MidiEvent],
        output: &mut [&mut [f32]],
    ) -> Result<(), HostAdapterError> {
        if output.len() != self.output_channels {
            return Err(HostAdapterError::ChannelCountMismatch {
                expected: self.output_channels,
                actual: output.len(),
            });
        }
        self.engine.set_controls(self.controls)?;
        self.engine.process_block_diffuse(events, output)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HostAdapterError {
    #[error("unsupported host state schema version {0}")]
    UnsupportedStateVersion(u32),
    #[error("invalid host state: {0}")]
    State(#[from] serde_json::Error),
    #[error("host output must have at least one channel")]
    InvalidOutputChannels,
    #[error("host supplied {actual} output channels; adapter was configured for {expected}")]
    ChannelCountMismatch { expected: usize, actual: usize },
    #[error(transparent)]
    Engine(#[from] M0EngineError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_twelve_tet_tuning;
    use particelle_midi::{MidiEventKind, NoteEvent};

    #[test]
    fn adapter_maps_controls_and_renders_planar_bus() {
        let mut adapter =
            M0HostAdapter::new(M0Config::default(), default_twelve_tet_tuning(), 4).unwrap();
        adapter.set_parameter(HostParameter::DelayMix, 0.3).unwrap();
        let event = MidiEvent {
            frame_offset: 0,
            kind: MidiEventKind::Note(NoteEvent {
                channel: 2,
                note: 60,
                velocity: 0.8,
                is_on: true,
            }),
        };
        let second = MidiEvent {
            frame_offset: 0,
            kind: MidiEventKind::Note(NoteEvent {
                channel: 10,
                note: 67,
                velocity: 0.8,
                is_on: true,
            }),
        };
        let mut a = [0.0; 64];
        let mut b = [0.0; 64];
        let mut c = [0.0; 64];
        let mut d = [0.0; 64];
        adapter
            .process_planar(&[event, second], &mut [&mut a, &mut b, &mut c, &mut d])
            .unwrap();
        assert!(a.iter().any(|sample| *sample != 0.0));
        assert_ne!(a, b);
        assert!(c.iter().any(|sample| *sample != 0.0));
        assert!(d.iter().any(|sample| *sample != 0.0));
    }

    #[test]
    fn rejected_parameter_does_not_change_active_controls() {
        let mut adapter =
            M0HostAdapter::new(M0Config::default(), default_twelve_tet_tuning(), 2).unwrap();
        let original = adapter.controls();
        assert!(adapter.set_parameter(HostParameter::DelayMix, 2.0).is_err());
        assert_eq!(adapter.controls(), original);
    }

    #[test]
    fn saved_session_restores_controls_and_audio() {
        let mut original =
            M0HostAdapter::new(M0Config::default(), default_twelve_tet_tuning(), 1).unwrap();
        original
            .set_parameter(HostParameter::BaseMorph, 0.9)
            .unwrap();
        original
            .set_parameter(HostParameter::DelayMix, 0.4)
            .unwrap();
        let state = original.save_state().unwrap();
        let mut restored =
            M0HostAdapter::new(M0Config::default(), default_twelve_tet_tuning(), 1).unwrap();
        restored.restore_state(&state).unwrap();
        assert_eq!(restored.controls(), original.controls());
        let event = particelle_midi::parse_midi_bytes(&[0x91, 60, 100], 0).unwrap();
        let mut a = [0.0; 512];
        let mut b = [0.0; 512];
        original
            .process_planar(std::slice::from_ref(&event), &mut [&mut a])
            .unwrap();
        restored.process_planar(&[event], &mut [&mut b]).unwrap();
        assert_eq!(a, b);
        assert!(b.iter().any(|sample| *sample != 0.0));
    }

    #[test]
    fn invalid_state_preserves_controls() {
        let mut adapter =
            M0HostAdapter::new(M0Config::default(), default_twelve_tet_tuning(), 2).unwrap();
        adapter
            .set_parameter(HostParameter::ReverbMix, 0.7)
            .unwrap();
        let original = adapter.controls();
        for state in [
            br#"{"schema_version":2,"base_morph":0.5,"modulation_hz":0.0,"delay_mix":0.0,"reverb_mix":0.0}"#.as_slice(),
            br#"{"schema_version":1,"base_morph":0.5,"modulation_hz":0.0,"delay_mix":2.0,"reverb_mix":0.0}"#.as_slice(),
            b"{}".as_slice(),
            b"invalid JSON".as_slice(),
        ] {
            assert!(adapter.restore_state(state).is_err());
            assert_eq!(adapter.controls(), original);
        }
    }

    #[test]
    fn transport_stop_releases_held_notes() {
        let mut adapter =
            M0HostAdapter::new(M0Config::default(), default_twelve_tet_tuning(), 1).unwrap();
        let event = particelle_midi::parse_midi_bytes(&[0x91, 60, 100], 0).unwrap();
        let mut output = [0.0; 512];
        adapter
            .process_planar(&[event], &mut [&mut output])
            .unwrap();
        adapter.all_notes_off();
        for _ in 0..1000 {
            adapter.process_planar(&[], &mut [&mut output]).unwrap();
        }
        assert_eq!(adapter.engine.active_voice_count(), 0);
        assert!(output.iter().all(|sample| sample.abs() < 1e-5));
    }
}
