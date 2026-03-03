/// Realtime MIDI ingest via `midir`.
///
/// This module is only compiled when the `realtime` feature is enabled.
/// It collects MIDI events off the audio thread and pushes them into a
/// lock-free ring buffer for consumption by the engine on the audio thread.
///
/// No MIDI parsing or event dispatch occurs on the audio thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use midir::{MidiInput, MidiInputConnection};
use ringbuf::traits::Producer;
use crate::events::MidiEvent;
use crate::routing::parse_midi_bytes;

/// Realtime MIDI host: opens a MIDI port and pushes events to a queue.
pub struct RealtimeMidiHost {
    running: Arc<AtomicBool>,
    connection: Option<MidiInputConnection<()>>,
}

impl RealtimeMidiHost {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            connection: None,
        }
    }

    /// Start listening on the named MIDI port.
    /// Events are pushed to the provided ring buffer producer.
    pub fn start<P: Producer<Item = MidiEvent> + Send + 'static>(
        &mut self,
        port_name: &str,
        mut producer: P
    ) -> Result<(), MidiHostError> {
        let mut midi_in = MidiInput::new("particelle-midi-in")
            .map_err(|e| MidiHostError::OpenFailed { reason: e.to_string() })?;
        midi_in.ignore(midir::Ignore::None);

        let ports = midi_in.ports();
        let mut selected_port = None;
        for port in ports.iter() {
            if let Ok(name) = midi_in.port_name(port) {
                if name == port_name {
                    selected_port = Some(port.clone());
                    break;
                }
            }
        }

        let port = selected_port.ok_or_else(|| MidiHostError::PortNotFound { name: port_name.to_string() })?;

        let running_flag = self.running.clone();

        let conn = midi_in.connect(&port, "particelle-realtime", move |_stamp, message, _| {
            if !running_flag.load(Ordering::Relaxed) {
                return;
            }
            // Parse event with a frame_offset of 0. Frame offsets for realtime
            // events can be calculated externally based on timing if necessary.
            if let Some(event) = parse_midi_bytes(message, 0) {
                let _ = producer.try_push(event); // try push, drop if ringbuffer is full
            }
        }, ()).map_err(|e| MidiHostError::OpenFailed { reason: e.to_string() })?;

        self.running.store(true, Ordering::Relaxed);
        self.connection = Some(conn);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.connection.take(); // Drops the connection
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MidiHostError {
    #[error("No MIDI port named '{name}'")]
    PortNotFound { name: String },
    #[error("Failed to open MIDI port: {reason}")]
    OpenFailed { reason: String },
}
