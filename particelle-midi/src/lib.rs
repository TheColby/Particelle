//! `particelle-midi` — MIDI and MPE ingest, routing, and event types.
//!
//! No MIDI parsing or dispatch occurs on the audio thread. Events are
//! collected off-thread and passed via a lock-free queue.

pub mod events;
pub mod mpe;
pub mod offline;
pub mod routing;

#[cfg(feature = "realtime")]
pub mod realtime;

pub use events::{ExpressionEvent, ExpressionKind, MidiEvent, MidiEventKind, NoteEvent};
pub use mpe::{MpeConfig, MpeVoiceState, MpeZone};
pub use offline::OfflineMidiReader;
pub use routing::{parse_midi_bytes, MidiRouter, RoutingRule};
