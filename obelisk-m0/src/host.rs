//! Framework-neutral block adapter for native M0 plug-in wrappers.
//!
//! VST3/AU glue owns format conversion and lifecycle; this module owns the
//! narrow contract that is identical across both: control changes occur at a
//! block boundary, MIDI is sample-offset event data, and output is planar.

use crate::{M0Config, M0Controls, M0Engine, M0EngineError};
use particelle_midi::MidiEvent;
use particelle_tuning::ScalaTuning;

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
        let mut a = [0.0; 64];
        let mut b = [0.0; 64];
        let mut c = [0.0; 64];
        let mut d = [0.0; 64];
        adapter
            .process_planar(&[event], &mut [&mut a, &mut b, &mut c, &mut d])
            .unwrap();
        assert!(a.iter().any(|sample| *sample != 0.0));
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(c, d);
    }

    #[test]
    fn rejected_parameter_does_not_change_active_controls() {
        let mut adapter =
            M0HostAdapter::new(M0Config::default(), default_twelve_tet_tuning(), 2).unwrap();
        let original = adapter.controls();
        assert!(adapter.set_parameter(HostParameter::DelayMix, 2.0).is_err());
        assert_eq!(adapter.controls(), original);
    }
}
