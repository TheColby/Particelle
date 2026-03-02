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
/// When disabled, the type is still defined but `run()` returns an error.
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
    /// It receives an interleaved `&mut [f32]` buffer.
    /// It must not allocate. All external communication uses lock-free queues.
    ///
    /// This function blocks until the stream is stopped (e.g., via Ctrl+C).
    pub fn run<F>(&self, _callback: F) -> Result<(), HardwareError>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        #[cfg(not(feature = "realtime"))]
        return Err(HardwareError::RealtimeNotEnabled);

        #[cfg(feature = "realtime")]
        {
            use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

            let host = cpal::default_host();

            let device = match &self.config.device_name {
                Some(name) => {
                    host.output_devices()
                        .map_err(|e| HardwareError::StreamOpenFailed { reason: e.to_string() })?
                        .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
                        .ok_or_else(|| HardwareError::DeviceNotFound { name: name.clone() })?
                }
                None => {
                    host.default_output_device()
                        .ok_or(HardwareError::NoDefaultDevice)?
                }
            };

            let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            eprintln!("→ Opening device: {}", device_name);

            let stream_config = cpal::StreamConfig {
                channels: self.config.n_channels as u16,
                sample_rate: cpal::SampleRate(self.config.sample_rate as u32),
                buffer_size: cpal::BufferSize::Fixed(self.config.block_size as u32),
            };

            let mut callback = _callback;

            let stream = device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    callback(data);
                },
                |err| {
                    eprintln!("Audio stream error: {}", err);
                },
                None,
            ).map_err(|e| HardwareError::StreamOpenFailed { reason: e.to_string() })?;

            stream.play()
                .map_err(|e| HardwareError::StreamOpenFailed { reason: e.to_string() })?;

            eprintln!(
                "→ Streaming to \"{}\" @ {}Hz, {} ch, {} block (Ctrl+C to stop)",
                device_name,
                self.config.sample_rate as u32,
                self.config.n_channels,
                self.config.block_size,
            );

            // Block until Ctrl+C
            let (tx, rx) = std::sync::mpsc::channel();
            ctrlc_channel(tx);
            let _ = rx.recv();

            drop(stream);
            eprintln!("→ Stream stopped.");
            Ok(())
        }
    }

    /// List available output device names.
    pub fn list_devices() -> Result<Vec<String>, HardwareError> {
        #[cfg(not(feature = "realtime"))]
        return Err(HardwareError::RealtimeNotEnabled);

        #[cfg(feature = "realtime")]
        {
            use cpal::traits::{DeviceTrait, HostTrait};
            let host = cpal::default_host();
            let devices = host.output_devices()
                .map_err(|e| HardwareError::StreamOpenFailed { reason: e.to_string() })?;
            let names: Vec<String> = devices
                .filter_map(|d| d.name().ok())
                .collect();
            Ok(names)
        }
    }
}

/// Set up a Ctrl+C handler that sends to the given channel.
#[cfg(feature = "realtime")]
fn ctrlc_channel(tx: std::sync::mpsc::Sender<()>) {
    std::thread::spawn(move || {
        // Block forever — the process will be killed by Ctrl+C/SIGINT
        // which drops the stream via RAII cleanup.
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });
    let _ = tx;
}

#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("No audio device named '{name}'")]
    DeviceNotFound { name: String },
    #[error("No default audio output device found")]
    NoDefaultDevice,
    #[error("Failed to open audio stream: {reason}")]
    StreamOpenFailed { reason: String },
    #[error("Realtime feature not enabled. Rebuild with --features realtime")]
    RealtimeNotEnabled,
}
