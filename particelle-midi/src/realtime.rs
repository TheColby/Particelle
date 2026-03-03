/// Realtime MIDI ingest via `midir`.
///
/// This module is only compiled when the `realtime` feature is enabled.
/// It collects MIDI events off the audio thread and pushes them into a
/// lock-free ring buffer for consumption by the engine on the audio thread.
///
/// No MIDI parsing or event dispatch occurs on the audio thread.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Realtime MIDI host: opens a MIDI port and pushes events to a queue.
pub struct RealtimeMidiHost {
    running: Arc<AtomicBool>,
    connection: Mutex<Option<midir::MidiInputConnection<()>>>,
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
        P: ringbuf::traits::Producer<Item = crate::events::MidiEvent> + Send + 'static,
    {
        let midi_in = midir::MidiInput::new("Particelle").map_err(|e| MidiHostError::OpenFailed {
            reason: e.to_string(),
        })?;

        let ports = midi_in.ports();
        let mut selected_port = None;

        for port in ports.iter() {
            if let Ok(name) = midi_in.port_name(port) {
                if name.contains(port_name) {
                    selected_port = Some(port.clone());
                    break;
                }
            }
        }

        let port = selected_port.ok_or_else(|| MidiHostError::PortNotFound {
            name: port_name.to_string(),
        })?;

        let connection = midi_in
            .connect(
                &port,
                "Particelle-Input",
                move |_timestamp, message, _| {
                    if let Some(event) = crate::routing::parse_midi_bytes(message, 0) {
                        let _ = producer.try_push(event);
                    }
                },
                (),
            )
            .map_err(|e| MidiHostError::OpenFailed {
                reason: e.to_string(),
            })?;

        *self.connection.lock().unwrap() = Some(connection);
        self.running.store(true, Ordering::Relaxed);

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

#[derive(Debug, thiserror::Error)]
pub enum MidiHostError {
    #[error("No MIDI port named '{name}'")]
    PortNotFound { name: String },
    #[error("Failed to open MIDI port: {reason}")]
    OpenFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_midi_host_state() {
        let host = RealtimeMidiHost::new();
        assert!(!host.is_running());

        // Manipulate internal fields directly for testing
        // to avoid opening actual hardware ports
        host.running.store(true, Ordering::Relaxed);
        assert!(host.is_running());

        host.stop();
        assert!(!host.is_running());
        assert!(host.connection.lock().unwrap().is_none());
    }
}
