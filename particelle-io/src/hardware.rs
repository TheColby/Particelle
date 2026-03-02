use thiserror::Error;

/// Hardware audio configuration.
#[derive(Debug, Clone)]
pub struct HardwareConfig {
    /// Device name to open, or `None` for the system default.
    pub device_name: Option<String>,
    /// Target output latency in milliseconds.
    pub latency_ms: f64,
    /// If true, open both input and output streams (duplex).
    pub duplex: bool,
    /// Number of output channels to request.
    pub n_channels: usize,
    /// Sample rate to request from the hardware driver.
    pub sample_rate: f64,
    /// Block size (frames per callback).
    pub block_size: usize,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            device_name: None,
            latency_ms: 10.0,
            duplex: false,
            n_channels: 2,
            sample_rate: 48000.0,
            block_size: 256,
        }
    }
}

/// Hardware audio host (CPAL-backed).
///
/// The `realtime` feature must be enabled to use this type.
/// When disabled, the type is still defined but `build()` returns an error.
pub struct HardwareHost {
    pub config: HardwareConfig,
}

impl HardwareHost {
    pub fn new(config: HardwareConfig) -> Self {
        Self { config }
    }

    /// Open the audio stream and begin processing.
    ///
    /// The provided callback runs on the audio thread.
    /// It must not allocate. All external communication uses lock-free queues.
    ///
    /// TODO: Phase 8 — implement with CPAL.
    pub fn run<F>(&self, _callback: F) -> Result<(), HardwareError>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        #[cfg(not(feature = "realtime"))]
        return Err(HardwareError::RealtimeNotEnabled);

        #[cfg(feature = "realtime")]
        {
            // TODO: enumerate CPAL devices, open stream, install callback
            Err(HardwareError::NotImplemented)
        }
    }

    /// List available output device names.
    pub fn list_devices() -> Result<Vec<String>, HardwareError> {
        #[cfg(not(feature = "realtime"))]
        return Err(HardwareError::RealtimeNotEnabled);

        #[cfg(feature = "realtime")]
        {
            // TODO: enumerate CPAL devices
            Ok(vec![])
        }
    }
}

#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("No audio device named '{name}'")]
    DeviceNotFound { name: String },
    #[error("Failed to open audio stream: {reason}")]
    StreamOpenFailed { reason: String },
    #[error("Realtime feature not enabled. Rebuild with --features realtime")]
    RealtimeNotEnabled,
    #[error("Hardware backend not yet implemented")]
    NotImplemented,
}
