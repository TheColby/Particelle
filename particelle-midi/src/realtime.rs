/// Realtime MIDI ingest via `midir`.
///
/// This module is only compiled when the `realtime` feature is enabled.
/// It collects MIDI events off the audio thread and pushes them into a
/// lock-free ring buffer for consumption by the engine on the audio thread.
///
/// No MIDI parsing or event dispatch occurs on the audio thread.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use midir::{MidiInput, MidiInputConnection};
use ringbuf::traits::Producer;
use crate::events::MidiEvent;
use crate::routing::parse_midi_bytes;

/// Realtime MIDI host: opens a MIDI port and pushes events to a queue.
pub struct RealtimeMidiHost {
    running: Arc<AtomicBool>,
    connection: Mutex<Option<MidiInputConnection<()>>>,
}

impl RealtimeMidiHost {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            connection: Mutex::new(None),
        }
    }

    /// Start listening on the named MIDI port.
    /// Events are pushed to the provided ring buffer producer.
    pub fn start<P>(&self, port_name: &str, mut producer: P) -> Result<(), MidiHostError>
    where
        P: Producer<Item = MidiEvent> + Send + 'static,
    {
        let mut midi_in = MidiInput::new("particelle-midi")
            .map_err(|e| MidiHostError::OpenFailed { reason: e.to_string() })?;

        midi_in.ignore(midir::Ignore::None);

        let ports = midi_in.ports();
        let port = ports.iter().find(|p| {
            midi_in.port_name(p).map(|n| n.contains(port_name)).unwrap_or(false)
        }).ok_or_else(|| MidiHostError::PortNotFound { name: port_name.to_string() })?;

        self.running.store(true, Ordering::Relaxed);

        let running_clone = self.running.clone();

        let conn = midi_in.connect(
            port,
            "particelle-input",
            move |_, message, _| {
                if !running_clone.load(Ordering::Relaxed) {
                    return;
                }

                if let Some(event) = parse_midi_bytes(message, 0) {
                    let _ = producer.try_push(event);
                }
            },
            ()
        ).map_err(|e| MidiHostError::OpenFailed { reason: e.to_string() })?;

        *self.connection.lock().unwrap() = Some(conn);

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        *self.connection.lock().unwrap() = None;
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Default for RealtimeMidiHost {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MidiHostError {
    #[error("No MIDI port named '{name}'")]
    PortNotFound { name: String },
    #[error("Failed to open MIDI port: {reason}")]
    OpenFailed { reason: String },
}
