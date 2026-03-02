use thiserror::Error;
use particelle_core::audio_block::AudioBlock;

/// Audio file reader.
///
/// Reads WAV files via `hound` and converts all sample formats to planar f64.
pub struct AudioFileReader {
    pub path: String,
    pub n_channels: usize,
    pub sample_rate: f64,
    pub n_frames: u64,
    /// All samples as planar f64 (channel-major).
    data: Vec<Vec<f64>>,
    /// Current read position in frames.
    read_pos: u64,
}

impl AudioFileReader {
    /// Open a WAV file for reading. Loads all samples into memory as planar f64.
    pub fn open(path: &str) -> Result<Self, FileError> {
        let reader = hound::WavReader::open(path)
            .map_err(|e| FileError::Read(format!("{}: {}", path, e)))?;

        let spec = reader.spec();
        let n_channels = spec.channels as usize;
        let sample_rate = spec.sample_rate as f64;

        // Read all samples into interleaved f64
        let interleaved: Vec<f64> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let max_val = (1i64 << (spec.bits_per_sample - 1)) as f64;
                reader.into_samples::<i32>()
                    .map(|s| s.map(|v| v as f64 / max_val))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| FileError::Read(e.to_string()))?
            }
            hound::SampleFormat::Float => {
                reader.into_samples::<f32>()
                    .map(|s| s.map(|v| v as f64))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| FileError::Read(e.to_string()))?
            }
        };

        let n_frames = interleaved.len() / n_channels;

        // De-interleave into planar format
        let mut data = vec![Vec::with_capacity(n_frames); n_channels];
        for (i, sample) in interleaved.iter().enumerate() {
            let ch = i % n_channels;
            data[ch].push(*sample);
        }

        Ok(Self {
            path: path.to_owned(),
            n_channels,
            sample_rate,
            n_frames: n_frames as u64,
            data,
            read_pos: 0,
        })
    }

    /// Read the next `block.frames` frames into `block`.
    /// Returns the number of frames actually read.
    pub fn read_block(&mut self, block: &mut AudioBlock) -> Result<usize, FileError> {
        let frames_remaining = self.n_frames.saturating_sub(self.read_pos) as usize;
        let frames_to_read = block.frames.min(frames_remaining);

        if frames_to_read == 0 {
            block.silence();
            return Ok(0);
        }

        let start = self.read_pos as usize;
        let end = start + frames_to_read;

        let n_ch = block.n_channels().min(self.n_channels);
        for ch in 0..n_ch {
            block.channels[ch][..frames_to_read].copy_from_slice(&self.data[ch][start..end]);
        }
        // Zero any extra channels in the block
        for ch in n_ch..block.n_channels() {
            for s in &mut block.channels[ch][..frames_to_read] {
                *s = 0.0;
            }
        }

        self.read_pos += frames_to_read as u64;
        Ok(frames_to_read)
    }

    /// Get the entire planar f64 data for a channel.
    pub fn channel_data(&self, ch: usize) -> Option<&[f64]> {
        self.data.get(ch).map(|v| v.as_slice())
    }

    /// Reset read position to the beginning.
    pub fn rewind(&mut self) {
        self.read_pos = 0;
    }
}

/// Audio file writer.
///
/// Writes WAV files via `hound`. Input is always planar f64;
/// conversion to the target bit depth (16, 24, or 32-bit float) occurs here.
pub struct AudioFileWriter {
    writer: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
    pub path: String,
    pub n_channels: usize,
    pub sample_rate: f64,
    pub bit_depth: u16,
    frames_written: u64,
}

impl AudioFileWriter {
    /// Create a new WAV file for writing.
    pub fn create(
        path: &str,
        n_channels: usize,
        sample_rate: f64,
        bit_depth: u16,
    ) -> Result<Self, FileError> {
        let sample_format = if bit_depth == 32 {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        };

        let spec = hound::WavSpec {
            channels: n_channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: bit_depth,
            sample_format,
        };

        let writer = hound::WavWriter::create(path, spec)
            .map_err(|e| FileError::Write(format!("{}: {}", path, e)))?;

        Ok(Self {
            writer,
            path: path.to_owned(),
            n_channels,
            sample_rate,
            bit_depth,
            frames_written: 0,
        })
    }

    /// Write a block of planar f64 samples. Interleaves and converts to target format.
    pub fn write_block(&mut self, block: &AudioBlock) -> Result<(), FileError> {
        let n_ch = block.n_channels().min(self.n_channels);
        let frames = block.frames;

        for f in 0..frames {
            for ch in 0..self.n_channels {
                let sample = if ch < n_ch { block.channels[ch][f] } else { 0.0 };
                match self.bit_depth {
                    16 => {
                        let val = (sample.clamp(-1.0, 1.0) * i16::MAX as f64) as i16;
                        self.writer.write_sample(val)
                            .map_err(|e| FileError::Write(e.to_string()))?;
                    }
                    24 => {
                        let val = (sample.clamp(-1.0, 1.0) * 8388607.0) as i32;
                        self.writer.write_sample(val)
                            .map_err(|e| FileError::Write(e.to_string()))?;
                    }
                    32 => {
                        let val = sample as f32;
                        self.writer.write_sample(val)
                            .map_err(|e| FileError::Write(e.to_string()))?;
                    }
                    _ => {
                        return Err(FileError::UnsupportedFormat(
                            format!("Unsupported bit depth: {}", self.bit_depth),
                        ));
                    }
                }
            }
        }

        self.frames_written += frames as u64;
        Ok(())
    }

    /// Finalize and close the WAV file (flushes the header).
    pub fn finalize(self) -> Result<u64, FileError> {
        let frames = self.frames_written;
        self.writer.finalize()
            .map_err(|e| FileError::Write(e.to_string()))?;
        Ok(frames)
    }
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("File not found: '{0}'")]
    NotFound(String),
    #[error("Unsupported format: '{0}'")]
    UnsupportedFormat(String),
    #[error("Read error: {0}")]
    Read(String),
    #[error("Write error: {0}")]
    Write(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read_wav() {
        let path = "/tmp/particelle_test_io.wav";
        let n_ch = 2;
        let sr = 44100.0;
        let frames = 1024;

        // Write a test WAV
        {
            let mut writer = AudioFileWriter::create(path, n_ch, sr, 16).unwrap();
            let mut block = AudioBlock::new(n_ch, frames);
            for i in 0..frames {
                block.channels[0][i] = 0.5;
                block.channels[1][i] = -0.5;
            }
            writer.write_block(&block).unwrap();
            writer.finalize().unwrap();
        }

        // Read it back
        {
            let reader = AudioFileReader::open(path).unwrap();
            assert_eq!(reader.n_channels, 2);
            assert!((reader.sample_rate - 44100.0).abs() < 1.0);
            assert_eq!(reader.n_frames, 1024);

            let left = reader.channel_data(0).unwrap();
            let right = reader.channel_data(1).unwrap();
            assert!((left[0] - 0.5).abs() < 0.001);
            assert!((right[0] - (-0.5)).abs() < 0.001);
        }

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_32bit_float() {
        let path = "/tmp/particelle_test_io_f32.wav";

        {
            let mut writer = AudioFileWriter::create(path, 1, 48000.0, 32).unwrap();
            let mut block = AudioBlock::new(1, 512);
            for i in 0..512 {
                block.channels[0][i] = (i as f64 / 512.0) * 2.0 - 1.0;
            }
            writer.write_block(&block).unwrap();
            writer.finalize().unwrap();
        }

        {
            let reader = AudioFileReader::open(path).unwrap();
            assert_eq!(reader.n_frames, 512);
            let data = reader.channel_data(0).unwrap();
            assert!((data[0] - (-1.0)).abs() < 0.001);
            assert!((data[256] - 0.0).abs() < 0.01);
        }

        std::fs::remove_file(path).ok();
    }
}
