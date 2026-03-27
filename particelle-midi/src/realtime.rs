//! Realtime MIDI ingest via `midir`.
//!
//! This module is only compiled when the `realtime` feature is enabled.
//! It collects MIDI events off the audio thread and forwards normalized
//! `MidiEvent`s through a channel for consumption on the audio thread.

use crate::events::MidiEvent;
use crate::routing::parse_midi_bytes;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Realtime MIDI host: opens one MIDI input port and forwards parsed events.
pub struct RealtimeMidiHost {
    running: Arc<AtomicBool>,
    connection: Option<midir::MidiInputConnection<()>>,
    selected_port: Option<String>,
}

impl RealtimeMidiHost {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            connection: None,
            selected_port: None,
        }
    }

    /// List available MIDI input port names.
    pub fn list_input_ports() -> Result<Vec<String>, MidiHostError> {
        let input = midir::MidiInput::new("particelle-midi-enumerate").map_err(|e| {
            MidiHostError::InitFailed {
                reason: e.to_string(),
            }
        })?;
        let mut names = Vec::new();
        for port in input.ports() {
            names.push(
                input
                    .port_name(&port)
                    .unwrap_or_else(|_| "Unknown MIDI port".to_string()),
            );
        }
        Ok(names)
    }

    /// Start listening on the selected MIDI input port.
    ///
    /// If `port_name` is `None`, the first available input port is used.
    /// Parsed events are forwarded through `tx`.
    pub fn start(
        &mut self,
        port_name: Option<&str>,
        tx: std::sync::mpsc::Sender<MidiEvent>,
    ) -> Result<String, MidiHostError> {
        if self.connection.is_some() {
            return Err(MidiHostError::AlreadyRunning);
        }

        let mut input = midir::MidiInput::new("particelle-midi-input").map_err(|e| {
            MidiHostError::InitFailed {
                reason: e.to_string(),
            }
        })?;
        input.ignore(midir::Ignore::None);

        let ports = input.ports();
        if ports.is_empty() {
            return Err(MidiHostError::NoInputPorts);
        }

        let selected_port = if let Some(requested_name) = port_name {
            let mut found = None;
            for port in ports {
                let name = input
                    .port_name(&port)
                    .unwrap_or_else(|_| "Unknown MIDI port".to_string());
                if name == requested_name {
                    found = Some(port);
                    break;
                }
            }
            found.ok_or_else(|| MidiHostError::PortNotFound {
                name: requested_name.to_string(),
            })?
        } else {
            ports[0].clone()
        };

        let selected_name = input
            .port_name(&selected_port)
            .unwrap_or_else(|_| "Unknown MIDI port".to_string());

        let running = Arc::clone(&self.running);
        let connection = input
            .connect(
                &selected_port,
                "particelle-midi-callback",
                move |_timestamp, message, _| {
                    if !running.load(Ordering::Relaxed) {
                        return;
                    }
                    if let Some(event) = parse_midi_bytes(message, 0) {
                        let _ = tx.send(event);
                    }
                },
                (),
            )
            .map_err(|e| MidiHostError::OpenFailed {
                reason: e.to_string(),
            })?;

        self.connection = Some(connection);
        self.selected_port = Some(selected_name.clone());
        self.running.store(true, Ordering::Relaxed);
        Ok(selected_name)
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.connection.take();
        self.selected_port = None;
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn selected_port(&self) -> Option<&str> {
        self.selected_port.as_deref()
    }
}

impl Default for RealtimeMidiHost {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MidiHostError {
    #[error("MIDI host is already running")]
    AlreadyRunning,
    #[error("No MIDI input ports are available")]
    NoInputPorts,
    #[error("No MIDI port named '{name}'")]
    PortNotFound { name: String },
    #[error("Failed to initialize MIDI host: {reason}")]
    InitFailed { reason: String },
    #[error("Failed to open MIDI port: {reason}")]
    OpenFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_not_running() {
        let host = RealtimeMidiHost::new();
        // The host should not be running immediately after creation
        assert!(!host.running.load(Ordering::Relaxed));
        assert!(!host.is_running());
    }
}
