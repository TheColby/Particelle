//! Realtime MIDI ingest via `midir`.
//!
//! This module is only compiled when the `realtime` feature is enabled.
//! It collects MIDI events off the audio thread and pushes them into a
//! lock-free ring buffer for consumption by the engine on the audio thread.
//!
//! No MIDI parsing or event dispatch occurs on the audio thread.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Realtime MIDI host: opens a MIDI port and pushes events to a queue.
pub struct RealtimeMidiHost {
    running: Arc<AtomicBool>,
}

impl RealtimeMidiHost {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start listening on the named MIDI port.
    /// Events are pushed to the provided ring buffer producer.
    ///
    /// TODO: Phase 7 — wire up `midir` port enumeration and callback.
    pub fn start(&self, _port_name: &str) -> Result<(), MidiHostError> {
        self.running.store(true, Ordering::Relaxed);
        // TODO: open midir port, install callback that writes to ring buffer
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
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
