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
        Self {
            frame: 0,
            config,
            layout,
        }
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
    pub fields: Box<dyn particelle_params::context::FieldProvider>,
}

impl GranularEngine {
    pub fn new(
        config: EngineConfig,
        layout: AudioLayout,
        spatializer: Box<dyn Spatializer>,
        fields: Box<dyn particelle_params::context::FieldProvider>,
    ) -> Result<Self, CoreError> {
        let state = EngineState::new(config, layout);
        Ok(Self {
            state,
            clouds: Vec::new(),
            spatializer,
            fields,
        })
    }

    pub fn add_cloud(&mut self, cloud: Cloud) {
        self.clouds.push(cloud);
    }
}

impl Engine for GranularEngine {
    fn process(&mut self, output: &mut AudioBlock) -> Result<(), CoreError> {
        output.silence();

        let sample_rate = self.state.config.sample_rate;
        let start_frame = self.state.frame;

        for cloud in &mut self.clouds {
            // For now, we update onset delay by the block size.
            // Better: loop sample-by-sample for accurate onsets.
            for i in 0..output.frames {
                let ctx = particelle_params::context::SignalContext {
                    frame: start_frame + i as crate::FrameCount,
                    sample_rate,
                    fields: self.fields.as_ref(),
                    custom_resolver: None,
                };
                cloud.update(sample_rate, self.spatializer.as_ref(), &ctx);
            }

            cloud.pool.process_all(output);
        }

        self.state.advance(output.frames);
        Ok(())
    }

    fn state(&self) -> &EngineState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grain::Cloud;
    use crate::pool::GrainPool;
    use crate::spatializer::AmplitudePanner;
    use std::sync::Arc;

    #[test]
    fn test_granular_engine_non_zero_output() -> Result<(), crate::error::CoreError> {
        let config = EngineConfig::new(44100.0, 256)?;
        let layout = crate::layout::AudioLayout::stereo();
        let panner = Box::new(AmplitudePanner::new(layout.clone()));
        let mut engine = GranularEngine::new(
            config,
            layout,
            panner,
            Box::new(particelle_params::context::NullFields),
        )?;

        // Dummy source: 1 second of 1.0 (constant signal)
        let source = Arc::new(vec![vec![1.0; 44100]]);
        let window = Arc::from(vec![1.0; 1024]); // Rectangular window for peak amplitude
        let pool = GrainPool::new(10, Arc::clone(&source), Arc::clone(&window), 2);
        let cloud = Cloud::new("test_cloud".to_string(), pool);
        engine.add_cloud(cloud);

        let mut output = AudioBlock::new(2, 256);
        engine.process(&mut output)?;

        // Verify that we have some non-zero samples in the output
        let mut has_sound = false;
        for ch in 0..output.n_channels() {
            for f in 0..output.frames {
                if output.channels[ch][f].abs() > 0.0 {
                    has_sound = true;
                    break;
                }
            }
        }

        assert!(has_sound, "Engine produced no sound (all samples are zero)");
        Ok(())
    }
}
