/// MIDI event types. All values are normalized to f64 ranges.
///
/// Raw MIDI values (0–127, 0–16383) are converted to f64 by the ingest layer.
/// No raw byte parsing occurs on the audio thread.

/// A complete MIDI event, timestamped in samples.
#[derive(Debug, Clone)]
pub struct MidiEvent {
    /// Frame offset within the current block (0 = block start).
    pub frame_offset: usize,
    pub kind: MidiEventKind,
}

#[derive(Debug, Clone)]
pub enum MidiEventKind {
    Note(NoteEvent),
    Expression(ExpressionEvent),
    PitchBend { channel: u8, value: f64 },        // [-1, 1]
    ChannelPressure { channel: u8, value: f64 },   // [0, 1]
    ProgramChange { channel: u8, program: u8 },
    ControlChange { channel: u8, cc: u8, value: f64 }, // [0, 1]
}

/// A note-on or note-off event.
#[derive(Debug, Clone)]
pub struct NoteEvent {
    pub channel: u8,
    pub note: u8,        // 0–127
    pub velocity: f64,   // [0, 1] (0.0 = note off)
    pub is_on: bool,
}

/// A per-note MPE expression event.
#[derive(Debug, Clone)]
pub struct ExpressionEvent {
    pub channel: u8,
    pub note: u8,
    pub kind: ExpressionKind,
    pub value: f64,      // [0, 1] for pressure/timbre, [-1, 1] for pitchbend
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    PitchBend,
    Pressure,
    Timbre,
}
