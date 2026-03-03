/// MPE (MIDI Polyphonic Expression) zone configuration and voice state.

/// MPE zone: defines the master and member channel range.
#[derive(Debug, Clone)]
pub struct MpeZone {
    /// Master channel (1-indexed). Typically channel 1 (lower zone) or 16 (upper zone).
    pub master_channel: u8,
    /// Member channels (1-indexed, inclusive range).
    pub member_channels: std::ops::RangeInclusive<u8>,
    /// Pitchbend range in semitones for member channels.
    pub pitchbend_range_semitones: f64,
}

impl MpeZone {
    /// Standard MPE lower zone: master on ch 1, members on ch 2–15.
    pub fn lower_zone(pitchbend_range_semitones: f64) -> Self {
        Self {
            master_channel: 1,
            member_channels: 2..=15,
            pitchbend_range_semitones,
        }
    }

    /// Standard MPE upper zone: master on ch 16, members on ch 2–15.
    pub fn upper_zone(pitchbend_range_semitones: f64) -> Self {
        Self {
            master_channel: 16,
            member_channels: 2..=15,
            pitchbend_range_semitones,
        }
    }

    pub fn is_member_channel(&self, ch: u8) -> bool {
        self.member_channels.contains(&ch)
    }
}

/// Per-voice MPE state, one per active note on a member channel.
#[derive(Debug, Clone, Default)]
pub struct MpeVoiceState {
    pub channel: u8,
    pub note: u8,
    pub pitchbend: f64, // [-1, 1]
    pub pressure: f64,  // [0, 1]
    pub timbre: f64,    // [0, 1] (CC74)
    pub velocity: f64,  // [0, 1]
    pub active: bool,
}

/// Global MPE configuration.
#[derive(Debug, Clone)]
pub struct MpeConfig {
    pub zones: Vec<MpeZone>,
    /// Maximum simultaneous voices.
    pub max_voices: usize,
}

impl Default for MpeConfig {
    fn default() -> Self {
        Self {
            zones: vec![MpeZone::lower_zone(48.0)],
            max_voices: 16,
        }
    }
}
