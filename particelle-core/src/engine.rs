use crate::audio_block::AudioBlock;
use crate::error::CoreError;
use crate::grain::Cloud;
use crate::layout::AudioLayout;
use crate::spatializer::Spatializer;
use crate::FrameCount;

/// Immutable engine configuration, fixed at initialization.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub sample_rate: f64,
    pub block_size: usize,
    pub max_particles_per_cloud: usize,
}

impl EngineConfig {
    pub fn new(sample_rate: f64, block_size: usize) -> Result<Self, CoreError> {
        if sample_rate <= 0.0 {
            return Err(CoreError::InvalidSampleRate { rate: sample_rate });
        }
        if block_size == 0 {
            return Err(CoreError::InvalidBlockSize { size: block_size });
        }
        Ok(Self {
            sample_rate,
            block_size,
            max_particles_per_cloud: 4096,
        })
    }
}

/// Current engine runtime state.
#[derive(Debug)]
pub struct EngineState {
    /// Monotonic sample-accurate frame counter. Never wraps during a session.
    pub frame: FrameCount,
    pub config: EngineConfig,
    pub layout: AudioLayout,
}

impl EngineState {
    pub fn new(config: EngineConfig, layout: AudioLayout) -> Self {
        Self { frame: 0, config, layout }
    }

    pub fn advance(&mut self, frames: usize) {
        self.frame += frames as FrameCount;
    }
}

/// The core processing trait. All engine implementations must satisfy this.
///
/// No implementation of `process` may allocate on the heap during realtime use.
pub trait Engine: Send {
    /// Fill `output` with one block of synthesized audio.
    fn process(&mut self, output: &mut AudioBlock) -> Result<(), CoreError>;
    /// Immutable access to the current engine state.
    fn state(&self) -> &EngineState;
}

/// Primary granular engine implementation.
pub struct GranularEngine {
    pub state: EngineState,
    pub clouds: Vec<Cloud>,
    pub spatializer: Box<dyn Spatializer>,
}

impl GranularEngine {
    pub fn new(config: EngineConfig, layout: AudioLayout, spatializer: Box<dyn Spatializer>) -> Result<Self, CoreError> {
        let state = EngineState::new(config, layout);
        Ok(Self {
            state,
            clouds: Vec::new(),
            spatializer,
        })
    }

    pub fn add_cloud(&mut self, cloud: Cloud) {
        self.clouds.push(cloud);
    }
}

impl Engine for GranularEngine {
    fn process(&mut self, output: &mut AudioBlock) -> Result<(), CoreError> {
        output.silence();
        // TODO: grain scheduling, window application, spatializer distribution
        self.state.advance(output.frames);
        Ok(())
    }

    fn state(&self) -> &EngineState {
        &self.state
    }
}
