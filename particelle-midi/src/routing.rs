use crate::events::*;
use std::collections::HashMap;

/// Routes incoming MIDI events to Field values in the signal graph.
///
/// Applied off the audio thread. Results are written into a field map that
/// the audio thread reads lock-free.
pub struct MidiRouter {
    pub rules: Vec<RoutingRule>,
}

/// Routing rule: maps a MIDI source path to a parameter field.
#[derive(Debug, Clone)]
pub struct RoutingRule {
    /// Source identifier, e.g. `"midi.cc.1"`, `"mpe.pressure"`, `"mpe.pitchbend"`.
    pub source: String,
    /// Target field name, e.g. `"density_mod"`, `"filter_cutoff"`.
    pub target: String,
    /// Affine transform: `output = input * scale + offset`.
    pub scale: f64,
    pub offset: f64,
}

impl RoutingRule {
    pub fn direct(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            scale: 1.0,
            offset: 0.0,
        }
    }

    pub fn with_transform(source: impl Into<String>, target: impl Into<String>, scale: f64, offset: f64) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            scale,
            offset,
        }
    }

    fn apply(&self, value: f64) -> f64 {
        value * self.scale + self.offset
    }
}

impl MidiRouter {
    pub fn new(rules: Vec<RoutingRule>) -> Self {
        Self { rules }
    }

    /// Process a batch of MIDI events and return a map of field name → value.
    ///
    /// If multiple events map to the same field within one block, the last value wins.
    pub fn process(&self, events: &[MidiEvent]) -> HashMap<String, f64> {
        let mut fields = HashMap::new();

        for event in events {
            let source_key = event_to_source_key(&event.kind);
            let value = event_to_value(&event.kind);

            for rule in &self.rules {
                if rule.source == source_key {
                    fields.insert(rule.target.clone(), rule.apply(value));
                }
            }
        }

        fields
    }
}

/// Convert a MidiEventKind to its canonical source path string.
fn event_to_source_key(kind: &MidiEventKind) -> String {
    match kind {
        MidiEventKind::ControlChange { cc, .. } => format!("midi.cc.{}", cc),
        MidiEventKind::PitchBend { .. } => "midi.pitchbend".to_string(),
        MidiEventKind::ChannelPressure { .. } => "midi.pressure".to_string(),
        MidiEventKind::Note(_) => "midi.note".to_string(),
        MidiEventKind::ProgramChange { .. } => "midi.program".to_string(),
        MidiEventKind::Expression(expr) => match expr.kind {
            ExpressionKind::PitchBend => "mpe.pitchbend".to_string(),
            ExpressionKind::Pressure => "mpe.pressure".to_string(),
            ExpressionKind::Timbre => "mpe.timbre".to_string(),
        },
    }
}

/// Extract the f64 value from a MidiEventKind.
fn event_to_value(kind: &MidiEventKind) -> f64 {
    match kind {
        MidiEventKind::ControlChange { value, .. } => *value,
        MidiEventKind::PitchBend { value, .. } => *value,
        MidiEventKind::ChannelPressure { value, .. } => *value,
        MidiEventKind::Note(n) => n.velocity,
        MidiEventKind::ProgramChange { program, .. } => *program as f64 / 127.0,
        MidiEventKind::Expression(expr) => expr.value,
    }
}

/// Parse raw MIDI bytes (3 bytes per message) into a MidiEvent.
///
/// Returns None for messages that are not relevant (e.g., system exclusive, timing clock).
pub fn parse_midi_bytes(bytes: &[u8], frame_offset: usize) -> Option<MidiEvent> {
    if bytes.is_empty() {
        return None;
    }
    let status = bytes[0];
    let msg_type = status & 0xF0;
    let channel = (status & 0x0F) + 1; // 1-indexed

    let kind = match msg_type {
        0x90 if bytes.len() >= 3 => {
            let note = bytes[1] & 0x7F;
            let vel = bytes[2] & 0x7F;
            let velocity = vel as f64 / 127.0;
            let is_on = vel > 0;
            MidiEventKind::Note(NoteEvent {
                channel,
                note,
                velocity,
                is_on,
            })
        }
        0x80 if bytes.len() >= 3 => {
            let note = bytes[1] & 0x7F;
            let vel = bytes[2] & 0x7F;
            MidiEventKind::Note(NoteEvent {
                channel,
                note,
                velocity: vel as f64 / 127.0,
                is_on: false,
            })
        }
        0xB0 if bytes.len() >= 3 => {
            let cc = bytes[1] & 0x7F;
            let val = bytes[2] & 0x7F;
            MidiEventKind::ControlChange {
                channel,
                cc,
                value: val as f64 / 127.0,
            }
        }
        0xE0 if bytes.len() >= 3 => {
            let lsb = bytes[1] & 0x7F;
            let msb = bytes[2] & 0x7F;
            let raw = ((msb as u16) << 7) | lsb as u16;
            let value = (raw as f64 - 8192.0) / 8192.0; // [-1, 1]
            MidiEventKind::PitchBend { channel, value }
        }
        0xD0 if bytes.len() >= 2 => {
            let val = bytes[1] & 0x7F;
            MidiEventKind::ChannelPressure {
                channel,
                value: val as f64 / 127.0,
            }
        }
        0xC0 if bytes.len() >= 2 => {
            let program = bytes[1] & 0x7F;
            MidiEventKind::ProgramChange { channel, program }
        }
        _ => return None,
    };

    Some(MidiEvent { frame_offset, kind })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_on() {
        let msg = [0x90, 60, 100]; // Ch1 NoteOn C4 vel=100
        let event = parse_midi_bytes(&msg, 0).unwrap();
        match &event.kind {
            MidiEventKind::Note(n) => {
                assert!(n.is_on);
                assert_eq!(n.channel, 1);
                assert_eq!(n.note, 60);
                assert!((n.velocity - 100.0 / 127.0).abs() < 1e-6);
            }
            _ => panic!("Expected NoteOn"),
        }
    }

    #[test]
    fn test_parse_note_off() {
        let msg = [0x80, 60, 64]; // Ch1 NoteOff C4
        let event = parse_midi_bytes(&msg, 0).unwrap();
        match &event.kind {
            MidiEventKind::Note(n) => assert!(!n.is_on),
            _ => panic!("Expected NoteOff"),
        }
    }

    #[test]
    fn test_parse_cc() {
        let msg = [0xB0, 1, 127]; // Ch1 CC1 max
        let event = parse_midi_bytes(&msg, 0).unwrap();
        match &event.kind {
            MidiEventKind::ControlChange { cc, value, .. } => {
                assert_eq!(*cc, 1);
                assert!((value - 1.0).abs() < 1e-6);
            }
            _ => panic!("Expected CC"),
        }
    }

    #[test]
    fn test_parse_pitchbend_center() {
        let msg = [0xE0, 0x00, 0x40]; // Center (8192)
        let event = parse_midi_bytes(&msg, 0).unwrap();
        match &event.kind {
            MidiEventKind::PitchBend { value, .. } => {
                assert!(value.abs() < 0.01);
            }
            _ => panic!("Expected PitchBend"),
        }
    }

    #[test]
    fn test_parse_channel_pressure() {
        let msg = [0xD0, 100]; // Ch1 pressure=100
        let event = parse_midi_bytes(&msg, 0).unwrap();
        match &event.kind {
            MidiEventKind::ChannelPressure { value, .. } => {
                assert!((value - 100.0 / 127.0).abs() < 1e-6);
            }
            _ => panic!("Expected ChannelPressure"),
        }
    }

    #[test]
    fn test_routing_cc_to_field() {
        let rules = vec![
            RoutingRule::direct("midi.cc.1", "density_mod"),
            RoutingRule::with_transform("midi.cc.74", "filter_cutoff", 20000.0, 20.0),
        ];
        let router = MidiRouter::new(rules);

        let events = vec![
            MidiEvent {
                frame_offset: 0,
                kind: MidiEventKind::ControlChange { channel: 1, cc: 1, value: 0.5 },
            },
            MidiEvent {
                frame_offset: 10,
                kind: MidiEventKind::ControlChange { channel: 1, cc: 74, value: 1.0 },
            },
        ];

        let fields = router.process(&events);
        assert!((fields["density_mod"] - 0.5).abs() < 1e-6);
        assert!((fields["filter_cutoff"] - 20020.0).abs() < 1e-6);
    }

    #[test]
    fn test_routing_mpe_pressure() {
        let rules = vec![RoutingRule::direct("mpe.pressure", "amplitude_mod")];
        let router = MidiRouter::new(rules);

        let events = vec![MidiEvent {
            frame_offset: 0,
            kind: MidiEventKind::Expression(ExpressionEvent {
                channel: 2,
                note: 60,
                kind: ExpressionKind::Pressure,
                value: 0.75,
            }),
        }];

        let fields = router.process(&events);
        assert!((fields["amplitude_mod"] - 0.75).abs() < 1e-6);
    }
}
