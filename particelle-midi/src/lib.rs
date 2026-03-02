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

pub use events::{MidiEvent, MidiEventKind, NoteEvent, ExpressionEvent, ExpressionKind};
pub use mpe::{MpeConfig, MpeZone, MpeVoiceState};
pub use offline::OfflineMidiReader;
pub use routing::{MidiRouter, RoutingRule, parse_midi_bytes};
