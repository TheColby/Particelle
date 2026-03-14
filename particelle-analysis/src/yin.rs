//! YIN Pitch Tracking Algorithm
//!
//! A rust implementation of the YIN fundamental frequency estimator by
//! De Cheveigné and Kawahara (2002). It is highly accurate for monophonic signals.

/// Configures the YIN pitch tracker.
#[derive(Debug, Clone, Copy)]
pub struct YinConfig {
    /// The threshold for the cumulative mean normalized difference function.
    /// Typically between 0.10 and 0.20. Lower is stricter (fewer false positives).
    pub threshold: f64,
    /// The minimum detectable frequency in Hz (determines maximum required integration window).
    pub min_freq: f64,
    /// The maximum detectable frequency in Hz.
    pub max_freq: f64,
    /// Audio sample rate
    pub sample_rate: f64,
}

impl Default for YinConfig {
    fn default() -> Self {
        Self {
            threshold: 0.15,
            min_freq: 40.0,
            max_freq: 2000.0,
            sample_rate: 48000.0,
        }
    }
}

/// A buffer for YIN analysis to avoid re-allocating on every frame.
pub struct YinBuffer {
    yin_buffer: Vec<f64>,
}

impl YinBuffer {
    /// Create a new YIN buffer sized for the given config.
    pub fn new(config: &YinConfig) -> Self {
        let max_tau = (config.sample_rate / config.min_freq).ceil() as usize;
        Self {
            yin_buffer: vec![0.0; max_tau],
        }
    }

    /// Estimate the fundamental frequency (f0) from a window of audio.
    /// The input `block` must be at least `2 * (sample_rate / min_freq)` samples long.
    /// Returns `Some(f0_hz)` if a pitch was detected below the threshold, or `None` if unpitched/noisy.
    pub fn estimate(&mut self, config: &YinConfig, block: &[f64]) -> Option<f64> {
        let max_tau = self.yin_buffer.len().min(block.len() / 2);
        if max_tau == 0 {
            return None;
        }

        // Step 1 & 2: Difference function
        self.difference(block, max_tau);

        // Step 3: Cumulative mean normalized difference function (CMND)
        self.cumulative_mean_normalized_difference(max_tau);

        // Step 4: Absolute threshold
        let tau_estimate = self.absolute_threshold(config.threshold, max_tau)?;

        // Step 5: Parabolic interpolation for sub-sample accuracy
        let refined_tau = self.parabolic_interpolation(tau_estimate, max_tau);

        let f0 = config.sample_rate / refined_tau;

        // Final sanity check on bounds
        if f0 >= config.min_freq && f0 <= config.max_freq {
            Some(f0)
        } else {
            None
        }
    }

    fn difference(&mut self, block: &[f64], max_tau: usize) {
        for tau in 0..max_tau {
            let mut delta = 0.0;
            for i in 0..max_tau {
                let diff = block[i] - block[i + tau];
                delta += diff * diff;
            }
            self.yin_buffer[tau] = delta;
        }
    }

    fn cumulative_mean_normalized_difference(&mut self, max_tau: usize) {
        self.yin_buffer[0] = 1.0;
        let mut running_sum = 0.0;

        for tau in 1..max_tau {
            running_sum += self.yin_buffer[tau];
            self.yin_buffer[tau] *= tau as f64 / running_sum;
        }
    }

    fn absolute_threshold(&self, threshold: f64, max_tau: usize) -> Option<usize> {
        let mut tau = 2; // skip tau 0 and 1
        while tau < max_tau {
            if self.yin_buffer[tau] < threshold {
                // Find the local minimum in this dip
                while tau + 1 < max_tau && self.yin_buffer[tau + 1] < self.yin_buffer[tau] {
                    tau += 1;
                }
                return Some(tau);
            }
            tau += 1;
        }
        None
    }

    fn parabolic_interpolation(&self, tau: usize, max_tau: usize) -> f64 {
        if tau == 0 || tau >= max_tau - 1 {
            return tau as f64;
        }

        let s0 = self.yin_buffer[tau - 1];
        let s1 = self.yin_buffer[tau];
        let s2 = self.yin_buffer[tau + 1];

        // y(x) = ax^2 + bx + c
        // Standard parabolic interpolation offset
        let adjustment = 0.5 * (s0 - s2) / (s0 - 2.0 * s1 + s2);

        tau as f64 + adjustment
    }
}
