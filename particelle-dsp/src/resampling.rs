use rubato::{Resampler as RubatoResamplerTrait, Fft, FixedSync};
use audioadapter::Adapter;
use audioadapter_buffers::direct::SequentialSliceOfVecs;

/// Resampler trait: converts a planar f64 buffer from one sample rate to another.
pub trait Resampler: Send {
    /// Resample `input` (at `input_rate` Hz) to approximately `output_rate` Hz.
    /// Returns a new planar buffer.
    fn resample(
        &mut self,
        input: &[Vec<f64>],
        input_rate: f64,
        output_rate: f64,
    ) -> Result<Vec<Vec<f64>>, String>;
}

/// A high-quality FFT-based resampler using `rubato`.
pub struct RubatoResampler {
    resampler: Option<Fft<f64>>,
    last_rates: (f64, f64),
    channels: usize,
}

impl RubatoResampler {
    pub fn new(channels: usize) -> Self {
        Self {
            resampler: None,
            last_rates: (0.0, 0.0),
            channels,
        }
    }
}

impl Resampler for RubatoResampler {
    fn resample(
        &mut self,
        input: &[Vec<f64>],
        input_rate: f64,
        output_rate: f64,
    ) -> Result<Vec<Vec<f64>>, String> {
        if input.is_empty() || input[0].is_empty() {
            return Ok(vec![vec![]; self.channels]);
        }

        if (input_rate - output_rate).abs() < 1e-6 {
            return Ok(input.to_vec()); // Fast path for no-op
        }

        let chunk_size = input[0].len();

        let need_new = self.resampler.is_none() || self.last_rates != (input_rate, output_rate);

        if need_new {
            self.resampler = Some(rubato::Fft::<f64>::new(
                input_rate as usize,
                output_rate as usize,
                chunk_size,
                1,
                self.channels,
                FixedSync::Input,
            ).map_err(|e| e.to_string())?);
            self.last_rates = (input_rate, output_rate);
        }

        let resampler = self.resampler.as_mut().unwrap();

        // Convert slice of Vecs to an Adapter
        // `SequentialSliceOfVecs` expects: slice of Vecs, channels, frames
        let frames_in = chunk_size;
        let multislice = SequentialSliceOfVecs::new(input, self.channels, frames_in)
            .map_err(|e| e.to_string())?;

        // Rubato expects buffers in a specific format
        let out = resampler.process(&multislice, 0, None).map_err(|e| e.to_string())?;

        // `InterleavedOwned` contains `channels` and interleaved data
        let frames = out.frames();
        let interleaved_data = out.take_data();
        let mut final_out = vec![vec![0.0; frames]; self.channels];
        
        for frame in 0..frames {
            for ch in 0..self.channels {
                final_out[ch][frame] = interleaved_data[frame * self.channels + ch];
            }
        }

        Ok(final_out)
    }
}
