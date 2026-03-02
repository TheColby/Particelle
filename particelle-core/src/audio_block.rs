use crate::error::CoreError;

/// Planar f64 multi-channel audio buffer.
///
/// One `Vec<f64>` per channel, all of length `frames`. Planar layout enables
/// per-channel processing without interleaving overhead. No implicit stereo
/// assumption exists anywhere in this type.
#[derive(Debug, Clone)]
pub struct AudioBlock {
    /// One sample buffer per channel.
    pub channels: Vec<Vec<f64>>,
    /// Number of frames (samples per channel) in this block.
    pub frames: usize,
}

impl AudioBlock {
    /// Allocate a new zeroed block. This is the only allocation point.
    /// Once constructed, realtime processing must not allocate.
    pub fn new(n_channels: usize, frames: usize) -> Self {
        Self {
            channels: vec![vec![0.0f64; frames]; n_channels],
            frames,
        }
    }

    /// Zero all samples in place. Does not reallocate.
    pub fn silence(&mut self) {
        for ch in &mut self.channels {
            ch.iter_mut().for_each(|s| *s = 0.0);
        }
    }

    /// Number of channels.
    pub fn n_channels(&self) -> usize {
        self.channels.len()
    }

    /// Mix `src` into `self` scaled by `gain`.
    /// Returns an error if channel counts or frame counts differ.
    pub fn mix_from(&mut self, src: &AudioBlock, gain: f64) -> Result<(), CoreError> {
        if src.n_channels() != self.n_channels() {
            return Err(CoreError::ChannelCountMismatch {
                expected: self.n_channels(),
                actual: src.n_channels(),
            });
        }
        if src.frames != self.frames {
            return Err(CoreError::BufferSizeMismatch {
                expected: self.frames,
                actual: src.frames,
            });
        }
        for (dst_ch, src_ch) in self.channels.iter_mut().zip(src.channels.iter()) {
            for (d, s) in dst_ch.iter_mut().zip(src_ch.iter()) {
                *d += s * gain;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_block_is_silent() {
        let block = AudioBlock::new(6, 256);
        assert_eq!(block.n_channels(), 6);
        assert_eq!(block.frames, 256);
        assert!(block.channels.iter().all(|ch| ch.iter().all(|&s| s == 0.0)));
    }

    #[test]
    fn silence_clears_samples() {
        let mut block = AudioBlock::new(2, 4);
        block.channels[0][0] = 1.0;
        block.silence();
        assert_eq!(block.channels[0][0], 0.0);
    }

    #[test]
    fn mix_from_channel_mismatch_errors() {
        let mut dst = AudioBlock::new(2, 4);
        let src = AudioBlock::new(6, 4);
        assert!(dst.mix_from(&src, 1.0).is_err());
    }
}
