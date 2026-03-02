use std::sync::Arc;
use crate::audio_block::AudioBlock;

/// State of a single active grain.
#[derive(Clone, Debug)]
pub struct Grain {
    /// Reference to the source audio buffer (planar f64).
    pub source: Arc<Vec<Vec<f64>>>,
    /// Current playback position in frames (at the original sample rate).
    pub current_frame: f64,
    /// Total duration of the grain in frames (at the output sample rate).
    pub duration_frames: f64,
    /// Number of frames rendered so far.
    pub rendered_frames: f64,
    /// Playback rate (pitch). 1.0 is original speed.
    pub playback_rate: f64,
    /// Pre-computed window function (e.g., Hann).
    pub window: Arc<[f64]>,
    /// Target speaker gains (panning).
    pub output_gains: Vec<f64>,
    /// Whether the grain is currently active.
    pub active: bool,
}

impl Grain {
    pub fn new(source: Arc<Vec<Vec<f64>>>, window: Arc<[f64]>, n_output_channels: usize) -> Self {
        Self {
            source,
            current_frame: 0.0,
            duration_frames: 0.0,
            rendered_frames: 0.0,
            playback_rate: 1.0,
            window,
            output_gains: vec![0.0; n_output_channels],
            active: false,
        }
    }

    pub fn activate(
        &mut self,
        start_frame: f64,
        duration_frames: f64,
        playback_rate: f64,
        gains: &[f64],
    ) {
        self.current_frame = start_frame;
        self.duration_frames = duration_frames;
        self.rendered_frames = 0.0;
        self.playback_rate = playback_rate;
        self.output_gains.copy_from_slice(gains);
        self.active = true;
    }

    /// Render into `output`. Returns `false` if the grain becomes inactive.
    pub fn process(&mut self, output: &mut AudioBlock) -> bool {
        if !self.active {
            return false;
        }

        let n_out_ch = output.n_channels().min(self.output_gains.len());
        let n_src_ch = self.source.len();
        let window_len = self.window.len() as f64;

        for f in 0..output.frames {
            if self.rendered_frames >= self.duration_frames {
                self.active = false;
                break;
            }

            // Window position [0.0, 1.0]
            let win_pos = self.rendered_frames / self.duration_frames;
            let win_idx = (win_pos * (window_len - 1.0)) as usize;
            let win_val = self.window[win_idx.min(self.window.len() - 1)];

            // Source position interpolation (linear)
            let src_idx = self.current_frame.floor() as usize;
            let src_fract = self.current_frame - (src_idx as f64);
            
            for ch in 0..n_src_ch {
                let src_buf = &self.source[ch];
                let s1 = src_buf.get(src_idx % src_buf.len()).copied().unwrap_or(0.0);
                let s2 = src_buf.get((src_idx + 1) % src_buf.len()).copied().unwrap_or(0.0);
                let sample = s1 + src_fract * (s2 - s1);
                
                let windowed = sample * win_val;

                // Apply panning gains to output channels
                for out_ch in 0..n_out_ch {
                    output.channels[out_ch][f] += windowed * self.output_gains[out_ch];
                }
            }

            self.current_frame += self.playback_rate;
            self.rendered_frames += 1.0;
        }

        self.active
    }
}

/// Parameters for emitting a new grain.
#[derive(Clone, Debug, Default)]
pub struct GrainParams {
    pub start_frame: f64,
    pub duration_frames: f64,
    pub playback_rate: f64,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub width: f64,
    pub amplitude: f64,
}

/// A high-level grain cloud managing a pool of grains.
pub struct Cloud {
    pub id: String,
    pub pool: crate::pool::GrainPool,
    /// Time until the next grain onset in frames.
    pub onset_delay: f64,
    
    // Interim parameters until Phase 12 (Signals)
    pub density: f64,
    pub duration: f64,
    pub amplitude: f64,
    pub position: f64,
    pub listener_pos: crate::spatializer::Vec3,
    pub width: f64,
    pub playback_rate: f64,
}

impl Cloud {
    pub fn new(id: String, pool: crate::pool::GrainPool) -> Self {
        Self {
            id,
            pool,
            onset_delay: 0.0,
            density: 10.0,
            duration: 0.1,
            amplitude: 0.5,
            position: 0.0,
            listener_pos: crate::spatializer::Vec3::ORIGIN,
            width: 0.5,
            playback_rate: 1.0,
        }
    }

    /// Update the cloud state and emit grains if necessary.
    pub fn update(
        &mut self,
        sample_rate: f64,
        spatializer: &dyn crate::spatializer::Spatializer,
    ) {
        if self.density <= 0.0 {
            return;
        }

        let avg_delay = sample_rate / self.density;
        
        if self.onset_delay <= 0.0 {
            if let Some(grain) = self.pool.acquire() {
                let mut gains = vec![0.0; grain.output_gains.len()];
                
                // Calculate spatial distribution
                spatializer.distribute(self.listener_pos, self.width, &mut gains);
                
                // Apply overall amplitude
                for g in &mut gains {
                    *g *= self.amplitude;
                }
                
                let start_frame = self.position * sample_rate; // simplistic mapping
                let dur_frames = self.duration * sample_rate;
                grain.activate(start_frame, dur_frames, self.playback_rate, &gains);
            }
            // Add some jitter to onset delay to avoid phasing (simplistic stochastic scheduler)
            // A real engine would use a noise generator here
            self.onset_delay = avg_delay;
        }
        self.onset_delay -= 1.0;
    }
}
