use crate::events::{ExpressionEvent, ExpressionKind, MidiEvent, MidiEventKind, NoteEvent};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use thiserror::Error;

/// Offline MIDI file reader. Produces a time-sorted list of `MidiEvent`s
/// from a standard MIDI file, suitable for deterministic offline rendering.
pub struct OfflineMidiReader {
    /// Time-stamped events with absolute frame positions.
    pub events: Vec<TimedMidiEvent>,
}

/// A MIDI event with an absolute frame position.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedMidiEvent {
    /// Absolute frame index at the engine's sample rate.
    pub frame: u64,
    pub event: MidiEvent,
}

/// Deterministic looping event harness for automated MIDI and MPE regression tests.
#[derive(Debug, Clone)]
pub struct DeterministicMidiHarness {
    pattern: Vec<TimedMidiEvent>,
    loop_length_frames: u64,
}

impl OfflineMidiReader {
    pub fn new(mut events: Vec<TimedMidiEvent>) -> Self {
        events.sort_by_key(|event| event.frame);
        Self { events }
    }

    /// Load and parse a MIDI file from raw bytes.
    ///
    /// Converts MIDI tick times to frame positions using the given
    /// `sample_rate` and tempo information from the file.
    pub fn from_bytes(data: &[u8], sample_rate: f64) -> Result<Self, MidiError> {
        let smf = Smf::parse(data).map_err(|err| MidiError::Parse(err.to_string()))?;
        let ticks_per_quarter = match smf.header.timing {
            Timing::Metrical(ticks) => u64::from(ticks.as_int()),
            Timing::Timecode(_, _) => return Err(MidiError::UnsupportedFormat),
        };

        let tempo_map = collect_tempo_map(&smf);
        let mut events = Vec::new();

        for track in &smf.tracks {
            let mut absolute_ticks = 0_u64;
            for event in track {
                absolute_ticks += u64::from(event.delta.as_int());
                let Some(kind) = parse_track_kind(&event.kind) else {
                    continue;
                };

                let seconds = ticks_to_seconds(absolute_ticks, &tempo_map, ticks_per_quarter);
                let frame = (seconds * sample_rate).round() as u64;
                events.push(TimedMidiEvent {
                    frame,
                    event: MidiEvent {
                        frame_offset: 0,
                        kind,
                    },
                });
            }
        }

        Ok(Self::new(events))
    }

    /// Events in the range [frame_start, frame_end).
    pub fn events_in_range(
        &self,
        frame_start: u64,
        frame_end: u64,
    ) -> impl Iterator<Item = &TimedMidiEvent> {
        self.events
            .iter()
            .filter(move |event| event.frame >= frame_start && event.frame < frame_end)
    }

    /// Clone events for one processing block, converting absolute frames into block offsets.
    pub fn block_events(&self, frame_start: u64, block_size: usize) -> Vec<MidiEvent> {
        let frame_end = frame_start + block_size as u64;
        self.events_in_range(frame_start, frame_end)
            .map(|timed| {
                let mut event = timed.event.clone();
                event.frame_offset = (timed.frame - frame_start) as usize;
                event
            })
            .collect()
    }
}

impl DeterministicMidiHarness {
    pub fn new(mut pattern: Vec<TimedMidiEvent>, loop_length_frames: u64) -> Self {
        assert!(loop_length_frames > 0, "loop length must be non-zero");
        pattern.sort_by_key(|event| event.frame);
        assert!(
            pattern.iter().all(|event| event.frame < loop_length_frames),
            "pattern event frame exceeds loop length"
        );
        Self {
            pattern,
            loop_length_frames,
        }
    }

    pub fn mpe_pressure_cycle(channel: u8, note: u8, step_frames: u64, values: &[f64]) -> Self {
        assert!(step_frames > 0, "step size must be non-zero");
        assert!(
            !values.is_empty(),
            "pressure cycle requires at least one value"
        );

        let mut pattern = Vec::with_capacity(values.len());
        for (idx, value) in values.iter().enumerate() {
            pattern.push(TimedMidiEvent {
                frame: idx as u64 * step_frames,
                event: MidiEvent {
                    frame_offset: 0,
                    kind: MidiEventKind::Expression(ExpressionEvent {
                        channel,
                        note,
                        kind: ExpressionKind::Pressure,
                        value: *value,
                    }),
                },
            });
        }

        Self::new(pattern, step_frames * values.len() as u64)
    }

    pub fn events_for_block(&self, frame_start: u64, block_size: usize) -> Vec<MidiEvent> {
        let frame_end = frame_start + block_size as u64;
        let first_cycle = frame_start / self.loop_length_frames;
        let last_cycle = frame_end.saturating_sub(1) / self.loop_length_frames;
        let mut events = Vec::new();

        for cycle in first_cycle..=last_cycle {
            let cycle_offset = cycle * self.loop_length_frames;
            for timed in &self.pattern {
                let absolute_frame = cycle_offset + timed.frame;
                if absolute_frame < frame_start || absolute_frame >= frame_end {
                    continue;
                }

                let mut event = timed.event.clone();
                event.frame_offset = (absolute_frame - frame_start) as usize;
                events.push(event);
            }
        }

        events.sort_by_key(|event| event.frame_offset);
        events
    }
}

fn collect_tempo_map(smf: &Smf<'_>) -> Vec<(u64, u32)> {
    let mut tempo_map = vec![(0, 500_000_u32)];

    for track in &smf.tracks {
        let mut absolute_ticks = 0_u64;
        for event in track {
            absolute_ticks += u64::from(event.delta.as_int());
            if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event.kind {
                tempo_map.push((absolute_ticks, tempo.as_int()));
            }
        }
    }

    tempo_map.sort_by_key(|(tick, _)| *tick);
    tempo_map.dedup_by_key(|(tick, _)| *tick);
    tempo_map
}

fn ticks_to_seconds(tick: u64, tempo_map: &[(u64, u32)], ticks_per_quarter: u64) -> f64 {
    let mut seconds = 0.0;
    let mut previous_tick = tempo_map[0].0;
    let mut current_tempo = tempo_map[0].1;

    for (change_tick, tempo) in tempo_map.iter().copied().skip(1) {
        if tick <= change_tick {
            break;
        }

        let delta_ticks = change_tick - previous_tick;
        seconds +=
            delta_ticks as f64 * current_tempo as f64 / 1_000_000.0 / ticks_per_quarter as f64;
        previous_tick = change_tick;
        current_tempo = tempo;
    }

    let remaining_ticks = tick.saturating_sub(previous_tick);
    seconds + remaining_ticks as f64 * current_tempo as f64 / 1_000_000.0 / ticks_per_quarter as f64
}

fn parse_track_kind(kind: &TrackEventKind<'_>) -> Option<MidiEventKind> {
    let TrackEventKind::Midi { channel, message } = kind else {
        return None;
    };

    let channel = channel.as_int() + 1;
    match message {
        MidiMessage::NoteOn { key, vel } => {
            let velocity = vel.as_int() as f64 / 127.0;
            Some(MidiEventKind::Note(NoteEvent {
                channel,
                note: key.as_int(),
                velocity,
                is_on: vel.as_int() > 0,
            }))
        }
        MidiMessage::NoteOff { key, vel } => Some(MidiEventKind::Note(NoteEvent {
            channel,
            note: key.as_int(),
            velocity: vel.as_int() as f64 / 127.0,
            is_on: false,
        })),
        MidiMessage::Aftertouch { vel, .. } => Some(MidiEventKind::ChannelPressure {
            channel,
            value: vel.as_int() as f64 / 127.0,
        }),
        MidiMessage::Controller { controller, value } => Some(MidiEventKind::ControlChange {
            channel,
            cc: controller.as_int(),
            value: value.as_int() as f64 / 127.0,
        }),
        MidiMessage::PitchBend { bend } => Some(MidiEventKind::PitchBend {
            channel,
            value: (bend.as_int() as f64 - 8192.0) / 8192.0,
        }),
        MidiMessage::ChannelAftertouch { vel } => Some(MidiEventKind::ChannelPressure {
            channel,
            value: vel.as_int() as f64 / 127.0,
        }),
        MidiMessage::ProgramChange { program } => Some(MidiEventKind::ProgramChange {
            channel,
            program: program.as_int(),
        }),
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

    #[test]
    fn offline_reader_maps_absolute_frames_to_block_offsets() {
        let reader = OfflineMidiReader::new(vec![
            TimedMidiEvent {
                frame: 16,
                event: MidiEvent {
                    frame_offset: 0,
                    kind: MidiEventKind::ControlChange {
                        channel: 1,
                        cc: 1,
                        value: 0.25,
                    },
                },
            },
            TimedMidiEvent {
                frame: 79,
                event: MidiEvent {
                    frame_offset: 0,
                    kind: MidiEventKind::ControlChange {
                        channel: 1,
                        cc: 74,
                        value: 0.75,
                    },
                },
            },
        ]);

        let block = reader.block_events(64, 64);
        assert_eq!(block.len(), 1);
        assert_eq!(block[0].frame_offset, 15);
        assert_eq!(
            block[0].kind,
            MidiEventKind::ControlChange {
                channel: 1,
                cc: 74,
                value: 0.75,
            }
        );
    }

    #[test]
    fn deterministic_harness_loops_events_across_block_boundaries() {
        let harness = DeterministicMidiHarness::new(
            vec![
                TimedMidiEvent {
                    frame: 0,
                    event: MidiEvent {
                        frame_offset: 0,
                        kind: MidiEventKind::Note(NoteEvent {
                            channel: 2,
                            note: 60,
                            velocity: 1.0,
                            is_on: true,
                        }),
                    },
                },
                TimedMidiEvent {
                    frame: 24,
                    event: MidiEvent {
                        frame_offset: 0,
                        kind: MidiEventKind::Expression(ExpressionEvent {
                            channel: 2,
                            note: 60,
                            kind: ExpressionKind::Pressure,
                            value: 0.5,
                        }),
                    },
                },
            ],
            32,
        );

        let block = harness.events_for_block(24, 16);
        assert_eq!(block.len(), 2);
        assert_eq!(block[0].frame_offset, 0);
        assert_eq!(block[1].frame_offset, 8);
        assert_eq!(
            block[1].kind,
            MidiEventKind::Note(NoteEvent {
                channel: 2,
                note: 60,
                velocity: 1.0,
                is_on: true,
            })
        );
    }

    #[test]
    fn pressure_cycle_emits_sorted_mpe_events() {
        let harness =
            DeterministicMidiHarness::mpe_pressure_cycle(2, 60, 64, &[0.0, 0.25, 0.5, 0.75]);
        let block = harness.events_for_block(64, 128);

        assert_eq!(block.len(), 2);
        assert_eq!(block[0].frame_offset, 0);
        assert_eq!(block[1].frame_offset, 64);

        match &block[0].kind {
            MidiEventKind::Expression(expr) => {
                assert_eq!(expr.kind, ExpressionKind::Pressure);
                assert!((expr.value - 0.25).abs() < 1e-9);
            }
            other => panic!("Expected pressure expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_standard_midi_bytes_into_timed_events() {
        let midi_bytes = [
            0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x00, 0x60,
            0x4D, 0x54, 0x72, 0x6B, 0x00, 0x00, 0x00, 0x13, 0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1,
            0x20, 0x00, 0x90, 0x3C, 0x64, 0x60, 0x80, 0x3C, 0x40, 0x00, 0xFF, 0x2F, 0x00,
        ];

        let reader = OfflineMidiReader::from_bytes(&midi_bytes, 48_000.0).unwrap();
        assert_eq!(reader.events.len(), 2);
        assert_eq!(reader.events[0].frame, 0);
        assert_eq!(reader.events[1].frame, 24_000);
        assert!(matches!(
            reader.events[0].event.kind,
            MidiEventKind::Note(NoteEvent { is_on: true, .. })
        ));
        assert!(matches!(
            reader.events[1].event.kind,
            MidiEventKind::Note(NoteEvent { is_on: false, .. })
        ));
    }
}
