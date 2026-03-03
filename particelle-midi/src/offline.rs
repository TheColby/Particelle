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
    pub fn events_in_range(
        &self,
        frame_start: u64,
        frame_end: u64,
    ) -> impl Iterator<Item = &TimedMidiEvent> {
        self.events
            .iter()
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
