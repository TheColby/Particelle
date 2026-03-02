use crate::grain::Grain;
use std::sync::Arc;

/// A fixed-size pool of pre-allocated grains.
pub struct GrainPool {
    pub grains: Vec<Grain>,
}

impl GrainPool {
    /// Create a new pool with `capacity` grains.
    pub fn new(
        capacity: usize,
        source: Arc<Vec<Vec<f64>>>,
        window: Arc<[f64]>,
        n_output_channels: usize,
    ) -> Self {
        let mut grains = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            grains.push(Grain::new(Arc::clone(&source), Arc::clone(&window), n_output_channels));
        }
        Self { grains }
    }

    /// Try to acquire an inactive grain from the pool.
    pub fn acquire(&mut self) -> Option<&mut Grain> {
        self.grains.iter_mut().find(|g| !g.active)
    }

    /// Update all active grains in the pool.
    pub fn process_all(&mut self, output: &mut crate::audio_block::AudioBlock) {
        for grain in &mut self.grains {
            if grain.active {
                grain.process(output);
            }
        }
    }
}
