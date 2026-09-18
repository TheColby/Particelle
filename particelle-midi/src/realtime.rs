//! Realtime MIDI ingest via `midir`.
//!
//! This module is only compiled when the `realtime` feature is enabled.
//! It collects MIDI events off the audio thread and forwards normalized
//! `MidiEvent`s through a channel for consumption on the audio thread.

use crate::events::MidiEvent;
use crate::routing::parse_midi_bytes;
use ringbuf::{traits::*, HeapCons, HeapRb};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
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
        self.start_with_handler(port_name, move |event| {
            let _ = tx.send(event);
        })
    }

    /// Start listening with a bounded lock-free queue suitable for an audio
    /// callback consumer. If the queue overflows, expression events are
    /// dropped and a lost note-off raises an emergency all-notes-off flag.
    pub fn start_ring(
        &mut self,
        port_name: Option<&str>,
        capacity: usize,
    ) -> Result<(String, RealtimeMidiQueue), MidiHostError> {
        if capacity == 0 {
            return Err(MidiHostError::InvalidQueueCapacity);
        }
        let ring = HeapRb::<MidiEvent>::new(capacity);
        let (mut producer, consumer) = ring.split();
        let dropped = Arc::new(AtomicU64::new(0));
        let emergency_all_notes_off = Arc::new(AtomicBool::new(false));
        let callback_dropped = Arc::clone(&dropped);
        let callback_emergency = Arc::clone(&emergency_all_notes_off);
        let selected = self.start_with_handler(port_name, move |event| {
            let lost_note_off = matches!(
                &event.kind,
                crate::events::MidiEventKind::Note(note) if !note.is_on
            );
            if producer.try_push(event).is_err() {
                callback_dropped.fetch_add(1, AtomicOrdering::Relaxed);
                if lost_note_off {
                    callback_emergency.store(true, Ordering::Release);
                }
            }
        })?;
        Ok((
            selected,
            RealtimeMidiQueue {
                consumer,
                dropped,
                emergency_all_notes_off,
            },
        ))
    }

    fn start_with_handler<F>(
        &mut self,
        port_name: Option<&str>,
        mut handler: F,
    ) -> Result<String, MidiHostError>
    where
        F: FnMut(MidiEvent) + Send + 'static,
    {
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
                        handler(event);
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

/// Audio-thread side of the bounded realtime MIDI queue.
pub struct RealtimeMidiQueue {
    consumer: HeapCons<MidiEvent>,
    dropped: Arc<AtomicU64>,
    emergency_all_notes_off: Arc<AtomicBool>,
}

impl RealtimeMidiQueue {
    pub fn try_pop(&mut self) -> Option<MidiEvent> {
        self.consumer.try_pop()
    }

    pub fn dropped_events(&self) -> u64 {
        self.dropped.load(AtomicOrdering::Relaxed)
    }

    pub fn take_emergency_all_notes_off(&self) -> bool {
        self.emergency_all_notes_off.swap(false, Ordering::AcqRel)
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
    #[error("Realtime MIDI queue capacity must be greater than zero")]
    InvalidQueueCapacity,
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
    use std::sync::atomic::Ordering;

    #[test]
    fn test_is_running() {
        let mut host = RealtimeMidiHost::new();

        // Initially, the host should not be running.
        assert!(!host.is_running());

        // Manually set the running flag to true to simulate starting.
        // We avoid calling start() to prevent attempting to open real MIDI ports in tests.
        host.running.store(true, Ordering::Relaxed);
        assert!(host.is_running());

        // Stopping the host should set it to not running.
        host.stop();
        assert!(!host.is_running());
    }
}
