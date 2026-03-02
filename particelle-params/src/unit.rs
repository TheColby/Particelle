/// Physical and musical units used by `ParamDescriptor`.
///
/// This enum is informational — it does not perform conversion. Unit
/// conversions are expressed as `MapFunc` nodes in the signal graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    /// No unit (dimensionless scalar).
    None,
    /// Normalized value, range hint [0, 1].
    Normalized,
    /// Decibels relative to full scale.
    DbFs,
    /// Linear amplitude (0.0 = silence, 1.0 = full scale).
    Linear,
    /// Hertz.
    Hz,
    /// Beats per minute.
    Bpm,
    /// Musical semitones.
    Semitones,
    /// Musical cents (1/100 semitone).
    Cents,
    /// Frequency ratio (e.g., 2.0 = one octave).
    Ratio,
    /// Seconds.
    Seconds,
    /// Milliseconds.
    Milliseconds,
    /// Sample frames.
    Frames,
    /// Degrees (azimuth, elevation).
    Degrees,
    /// Meters (spatial position).
    Meters,
    /// MIDI note number (0–127).
    MidiNote,
    /// MIDI channel (1–16 or MPE zone).
    MidiChannel,
}
