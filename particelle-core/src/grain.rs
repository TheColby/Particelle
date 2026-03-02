use crate::spatializer::Vec3;

/// A single active grain (particle).
///
/// All fields are set at grain birth and updated each block. No allocation
/// occurs after grain creation. Grains are managed in a pre-allocated pool.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Sub-sample-accurate read position into the Matter buffer.
    pub read_pos: f64,
    /// Playback rate relative to the original sample rate (1.0 = unmodified).
    pub rate: f64,
    /// Grain duration in frames.
    pub duration_frames: usize,
    /// Number of frames elapsed since grain birth.
    pub elapsed_frames: usize,
    /// Grain position in 3D listener space.
    pub position: Vec3,
    /// Spatial spread parameter [0, 1].
    pub width: f64,
    /// Pre-computed per-channel gain values (computed by Spatializer at birth).
    pub gains: Vec<f64>,
    /// Amplitude scalar.
    pub amplitude: f64,
    /// Opaque window identifier, resolved from the window cache.
    pub window_id: u64,
}

impl Particle {
    /// Returns true if the grain still has frames remaining.
    pub fn is_alive(&self) -> bool {
        self.elapsed_frames < self.duration_frames
    }

    /// Normalized window phase in [0, 1].
    pub fn window_phase(&self) -> f64 {
        if self.duration_frames == 0 {
            return 1.0;
        }
        self.elapsed_frames as f64 / self.duration_frames as f64
    }
}

/// Parameters controlling grain emission for one cloud.
///
/// All fields here are evaluated from `ParamSignal` in the engine; these
/// are the resolved f64 values at a given block boundary.
#[derive(Debug, Clone)]
pub struct EmitterParams {
    /// Grains per second.
    pub density: f64,
    /// Grain duration in seconds.
    pub duration_s: f64,
    /// Normalized read position into the Matter buffer [0, 1].
    pub position: f64,
    /// Position scatter, in units of duration [0, 1].
    pub position_scatter: f64,
    /// Playback rate ratio (1.0 = original pitch).
    pub rate: f64,
    /// Rate scatter in semitones.
    pub rate_scatter_semitones: f64,
    /// Peak amplitude.
    pub amplitude: f64,
    /// 3D position in listener space.
    pub listener_pos: Vec3,
    /// Spatial spread [0, 1].
    pub width: f64,
}

/// A cloud of grains sharing the same source material and emitter configuration.
///
/// The cloud owns a pool of `Particle` slots pre-allocated at construction.
/// The engine scheduler activates and retires particles without allocation.
#[derive(Debug)]
pub struct Cloud {
    pub id: u64,
    pub params: EmitterParams,
    /// Active particle pool. Pre-allocated at cloud creation.
    pub active_particles: Vec<Particle>,
    /// Maximum simultaneous particles this cloud may sustain.
    pub max_particles: usize,
    /// Fractional grain counter — tracks sub-grain scheduling debt.
    pub grain_debt: f64,
}

impl Cloud {
    pub fn new(id: u64, max_particles: usize, params: EmitterParams) -> Self {
        Self {
            id,
            params,
            active_particles: Vec::with_capacity(max_particles),
            max_particles,
            grain_debt: 0.0,
        }
    }

    pub fn particle_count(&self) -> usize {
        self.active_particles.len()
    }

    pub fn is_full(&self) -> bool {
        self.active_particles.len() >= self.max_particles
    }
}
