//! Browser-safe, hardware-free offline rendering entry point.

use crate::M0Engine;
use particelle_midi::MidiEvent;

/// Render one MIDI block and return interleaved stereo samples. This API has
/// no device, filesystem, or thread dependency, so it can be wrapped by WASM.
pub fn render_interleaved_stereo(
    engine: &mut M0Engine,
    events: &[MidiEvent],
    frames: usize,
) -> Vec<f32> {
    let mut stereo = vec![[0.0; 2]; frames];
    engine.process_block(events, &mut stereo);
    stereo.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{default_twelve_tet_tuning, M0Config};
    use particelle_midi::{MidiEventKind, NoteEvent};

    #[test]
    fn offline_render_is_interleaved_and_audible() {
        let mut engine = M0Engine::new(M0Config::default(), default_twelve_tet_tuning()).unwrap();
        let event = MidiEvent {
            frame_offset: 0,
            kind: MidiEventKind::Note(NoteEvent {
                channel: 2,
                note: 60,
                velocity: 0.8,
                is_on: true,
            }),
        };
        let output = render_interleaved_stereo(&mut engine, &[event], 128);
        assert_eq!(output.len(), 256);
        assert!(output.iter().any(|sample| *sample != 0.0));
    }
}
