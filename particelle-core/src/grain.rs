use crate::audio_block::AudioBlock;
use std::sync::Arc;

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
    /// Wrap source positions and interpolation at source boundaries.
    pub loop_source: bool,
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
            loop_source: true,
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
        self.output_gains.copy_from_slice(gains);
        self.activate_with_current_gains(start_frame, duration_frames, playback_rate);
    }

    fn activate_with_current_gains(
        &mut self,
        start_frame: f64,
        duration_frames: f64,
        playback_rate: f64,
    ) {
        self.current_frame = start_frame;
        self.duration_frames = duration_frames;
        self.rendered_frames = 0.0;
        self.playback_rate = playback_rate;
        self.active = true;
    }

    /// Render into `output`. Returns `false` if the grain becomes inactive.
    pub fn process(&mut self, output: &mut AudioBlock) -> bool {
        self.process_range(output, 0..output.frames)
    }

    pub(crate) fn process_range(
        &mut self,
        output: &mut AudioBlock,
        frames: std::ops::Range<usize>,
    ) -> bool {
        if !self.active {
            return false;
        }

        // Ragged planar input ends at the shortest channel; empty input is silent.
        let source_len = self.source.iter().map(Vec::len).min().unwrap_or(0);
        if source_len == 0
            || self.window.is_empty()
            || !self.current_frame.is_finite()
            || !self.playback_rate.is_finite()
            || !self.duration_frames.is_finite()
            || self.duration_frames <= 0.0
        {
            self.active = false;
            return false;
        }

        let n_out_ch = output.n_channels().min(self.output_gains.len());
        let n_src_ch = self.source.len();
        let window_len = self.window.len() as f64;

        for f in frames {
            if self.rendered_frames >= self.duration_frames
                || !self.current_frame.is_finite()
                || (!self.loop_source && !(0.0..source_len as f64).contains(&self.current_frame))
            {
                self.active = false;
                break;
            }

            // Window position [0.0, 1.0]
            let win_pos = self.rendered_frames / self.duration_frames;
            let win_idx = (win_pos * (window_len - 1.0)) as usize;
            let win_val = self.window[win_idx.min(self.window.len() - 1)];

            // Source position interpolation (linear)
            if self.loop_source {
                self.current_frame = self.current_frame.rem_euclid(source_len as f64);
            }
            let src_idx = (self.current_frame.floor() as usize).min(source_len - 1);
            let src_fract = self.current_frame - src_idx as f64;
            let next_idx = if src_idx + 1 < source_len {
                src_idx + 1
            } else if self.loop_source {
                0
            } else {
                src_idx // Hold the final sample; never interpolate across a nonlooping edge.
            };

            let mut mixed_sample = 0.0;
            for ch in 0..n_src_ch {
                let src_buf = &self.source[ch];
                let s1 = src_buf[src_idx];
                let s2 = src_buf[next_idx];
                mixed_sample += s1 + src_fract * (s2 - s1);
            }
            if n_src_ch > 1 {
                mixed_sample /= n_src_ch as f64;
            }

            let windowed = mixed_sample * win_val;

            // Apply panning gains to output channels
            for out_ch in 0..n_out_ch {
                output.channels[out_ch][f] += windowed * self.output_gains[out_ch];
            }

            // Reduce the step before adding so a finite, very large loop rate
            // cannot overflow the position and silence the grain.
            self.current_frame += if self.loop_source {
                self.playback_rate.rem_euclid(source_len as f64)
            } else {
                self.playback_rate
            };
            self.rendered_frames += 1.0;
        }

        // Release immediately, including when the last sample is a block/span boundary.
        if self.rendered_frames >= self.duration_frames
            || !self.current_frame.is_finite()
            || (!self.loop_source && !(0.0..source_len as f64).contains(&self.current_frame))
        {
            self.active = false;
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

use particelle_params::context::SignalContext;
use particelle_params::signal::ParamSignal;

/// A high-level grain cloud managing a pool of grains.
pub struct Cloud {
    pub id: String,
    pub pool: crate::pool::GrainPool,
    /// Time until the next grain onset in frames.
    pub onset_delay: f64,

    // Dynamic Parameter Signals
    pub density: ParamSignal,
    pub duration: ParamSignal,
    pub amplitude: ParamSignal,
    /// Source position in seconds, converted using `source_sample_rate` when set.
    pub position: ParamSignal,
    /// Source frames per second. `None` uses the engine rate for legacy callers.
    pub source_sample_rate: Option<f64>,
    // Spatialization params
    pub listener_pos: crate::spatializer::Vec3,
    pub width: ParamSignal,
    pub playback_rate: ParamSignal,
    /// New grains wrap their source by default. Existing grains retain their setting.
    pub loop_source: bool,
    pub directivity: ParamSignal,
    pub orientation_azimuth: ParamSignal,
    pub orientation_elevation: ParamSignal,
}

impl Cloud {
    pub fn new(id: String, pool: crate::pool::GrainPool) -> Self {
        Self {
            id,
            pool,
            onset_delay: 0.0,
            density: ParamSignal::Const(10.0),
            duration: ParamSignal::Const(0.1),
            amplitude: ParamSignal::Const(0.5),
            position: ParamSignal::Const(0.0),
            source_sample_rate: None,
            listener_pos: crate::spatializer::Vec3::ORIGIN,
            width: ParamSignal::Const(0.5),
            playback_rate: ParamSignal::Const(1.0),
            loop_source: true,
            directivity: ParamSignal::Const(1.0),
            orientation_azimuth: ParamSignal::Const(0.0),
            orientation_elevation: ParamSignal::Const(0.0),
        }
    }

    pub fn update(
        &mut self,
        sample_rate: f64,
        spatializer: &dyn crate::spatializer::Spatializer,
        ctx: &SignalContext<'_>,
    ) {
        let density_val = self.density.eval(ctx);
        self.update_with_density(sample_rate, spatializer, ctx, density_val);
    }

    pub(crate) fn update_with_density(
        &mut self,
        sample_rate: f64,
        spatializer: &dyn crate::spatializer::Spatializer,
        ctx: &SignalContext<'_>,
        density_val: f64,
    ) {
        if !density_val.is_finite() || density_val <= 0.0 {
            return;
        }

        // At most one onset per sample, retaining sub-sample residuals below Nyquist.
        let avg_delay = (sample_rate / density_val).max(1.0);

        if self.onset_delay <= 0.0 {
            if let Some(grain) = self.pool.acquire() {
                grain.output_gains.fill(0.0);

                let width_val = self.width.eval(ctx);
                let amplitude_val = self.amplitude.eval(ctx);
                let directivity_val = self.directivity.eval(ctx).clamp(0.0, 1.0);
                let az = self.orientation_azimuth.eval(ctx);
                let el = self.orientation_elevation.eval(ctx);
                let orientation = crate::spatializer::Vec3::from_az_el(az, el);

                // Calculate spatial distribution
                spatializer.distribute(
                    self.listener_pos,
                    orientation,
                    directivity_val,
                    width_val,
                    &mut grain.output_gains,
                );

                // Apply overall amplitude
                for g in &mut grain.output_gains {
                    *g *= amplitude_val;
                }

                let position_val = self.position.eval(ctx);
                let duration_val = self.duration.eval(ctx);
                let playback_rate_val = self.playback_rate.eval(ctx);

                let source_rate = self
                    .source_sample_rate
                    .filter(|rate| rate.is_finite() && *rate > 0.0)
                    .unwrap_or(sample_rate);
                let start_frame = position_val * source_rate;
                let dur_frames = duration_val * sample_rate;
                grain.loop_source = self.loop_source;
                grain.activate_with_current_gains(start_frame, dur_frames, playback_rate_val);
            }
            self.onset_delay += avg_delay;
        }
        self.onset_delay -= 1.0;
    }
}
