/// Realtime MIDI ingest via `midir`.
///
/// This module is only compiled when the `realtime` feature is enabled.
/// It collects MIDI events off the audio thread and pushes them into a
/// lock-free ring buffer for consumption by the engine on the audio thread.
///
/// No MIDI parsing or event dispatch occurs on the audio thread.

use midir::{MidiInput, MidiInputConnection};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

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
    ///
    /// TODO: Phase 7 — wire up `midir` port enumeration and callback.
    pub fn start<P>(&self, port_name: &str, mut producer: P) -> Result<(), MidiHostError>
    where
        P: ringbuf::traits::Producer<Item = crate::events::MidiEvent> + Send + 'static,
    {
        let mut midi_in = MidiInput::new("Particelle Realtime")
            .map_err(|e| MidiHostError::OpenFailed { reason: e.to_string() })?;

        midi_in.ignore(midir::Ignore::None);

        let ports = midi_in.ports();
        let mut target_port = None;

        for port in ports {
            if let Ok(name) = midi_in.port_name(&port) {
                if name == port_name {
                    target_port = Some(port);
                    break;
                }
            }
        }

        let port = target_port.ok_or_else(|| MidiHostError::PortNotFound {
            name: port_name.to_string(),
        })?;

        let connection = midi_in
            .connect(
                &port,
                "particelle-in",
                move |_, bytes: &[u8], _| {
                    if let Some(event) = crate::routing::parse_midi_bytes(bytes, 0) {
                        let _ = producer.try_push(event);
                    }
                },
                (),
            )
            .map_err(|e| MidiHostError::OpenFailed { reason: e.to_string() })?;

        *self.connection.lock().unwrap() = Some(connection);
        self.running.store(true, Ordering::Relaxed);

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        let mut conn_guard = self.connection.lock().unwrap();
        *conn_guard = None;
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
