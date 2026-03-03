use crate::events::MidiEvent;
use thiserror::Error;

/// Offline MIDI file reader. Produces a time-sorted list of `MidiEvent`s
/// from a standard MIDI file, suitable for deterministic offline rendering.
///
/// Uses the `midly` crate for parsing. No realtime feature required.
pub struct OfflineMidiReader {
    /// Time-stamped events with absolute frame positions.
    pub events: Vec<TimedMidiEvent>,
}

/// A MIDI event with an absolute frame position.
#[derive(Debug, Clone)]
pub struct TimedMidiEvent {
    /// Absolute frame index at the engine's sample rate.
    pub frame: u64,
    pub event: MidiEvent,
}

impl OfflineMidiReader {
    /// Load and parse a MIDI file from raw bytes.
    ///
    /// Converts MIDI tick times to frame positions using the given
    /// `sample_rate` and tempo information from the file.
    pub fn from_bytes(data: &[u8], sample_rate: f64) -> Result<Self, MidiError> {
        // TODO: Phase 7 — parse with `midly`, convert ticks to frames
        let _ = (data, sample_rate);
        Ok(Self { events: vec![] })
    }

    /// Events in the range [frame_start, frame_end).
    pub fn events_in_range(&self, frame_start: u64, frame_end: u64) -> impl Iterator<Item = &TimedMidiEvent> {
        self.events.iter()
            .filter(move |e| e.frame >= frame_start && e.frame < frame_end)
    }
}

#[derive(Debug, Error)]
pub enum MidiError {
    #[error("MIDI parse error: {0}")]
    Parse(String),
    #[error("No tempo information found in MIDI file")]
    NoTempo,
    #[error("Unsupported MIDI format")]
    UnsupportedFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{MidiEvent, MidiEventKind, NoteEvent};

    fn create_dummy_event(frame: u64) -> TimedMidiEvent {
        TimedMidiEvent {
            frame,
            event: MidiEvent {
                frame_offset: 0,
                kind: MidiEventKind::Note(NoteEvent {
                    channel: 0,
                    note: 60,
                    velocity: 0.5,
                    is_on: true,
                }),
            },
        }
    }

    #[test]
    fn test_events_in_range() {
        let mut reader = OfflineMidiReader { events: vec![] };

        // Add events at frames 10, 20, 30, 40, 50
        reader.events.push(create_dummy_event(10));
        reader.events.push(create_dummy_event(20));
        reader.events.push(create_dummy_event(30));
        reader.events.push(create_dummy_event(40));
        reader.events.push(create_dummy_event(50));

        // Test range fully enclosing some events: [15, 45) -> expects 20, 30, 40
        let in_range: Vec<_> = reader.events_in_range(15, 45).collect();
        assert_eq!(in_range.len(), 3);
        assert_eq!(in_range[0].frame, 20);
        assert_eq!(in_range[1].frame, 30);
        assert_eq!(in_range[2].frame, 40);

        // Test exact boundaries: [20, 40) -> expects 20, 30 (end is exclusive)
        let exact: Vec<_> = reader.events_in_range(20, 40).collect();
        assert_eq!(exact.len(), 2);
        assert_eq!(exact[0].frame, 20);
        assert_eq!(exact[1].frame, 30);

        // Test single point boundary: [30, 30) -> expects none
        let empty: Vec<_> = reader.events_in_range(30, 30).collect();
        assert_eq!(empty.len(), 0);

        // Test range completely before all events: [0, 5) -> expects none
        let before: Vec<_> = reader.events_in_range(0, 5).collect();
        assert_eq!(before.len(), 0);

        // Test range completely after all events: [60, 100) -> expects none
        let after: Vec<_> = reader.events_in_range(60, 100).collect();
        assert_eq!(after.len(), 0);

        // Test range containing all events: [0, 100) -> expects all
        let all: Vec<_> = reader.events_in_range(0, 100).collect();
        assert_eq!(all.len(), 5);
    }
}
