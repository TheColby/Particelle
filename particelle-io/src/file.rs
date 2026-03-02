use thiserror::Error;
use particelle_core::audio_block::AudioBlock;

/// Audio file reader interface (stub).
///
/// Phase 8 will implement WAV/FLAC reading via `hound`/`claxon`.
/// The interface reads planar f64 regardless of the source bit depth.
pub struct AudioFileReader {
    pub path: String,
    pub n_channels: usize,
    pub sample_rate: f64,
    pub n_frames: u64,
}

impl AudioFileReader {
    /// Open an audio file for reading.
    /// TODO: Phase 8 — implement with `hound` for WAV, `claxon` for FLAC.
    pub fn open(path: &str) -> Result<Self, FileError> {
        let _ = path;
        Err(FileError::NotImplemented("AudioFileReader::open".to_owned()))
    }

    /// Read the next `block.frames` frames into `block`.
    pub fn read_block(&mut self, _block: &mut AudioBlock) -> Result<usize, FileError> {
        Err(FileError::NotImplemented("AudioFileReader::read_block".to_owned()))
    }
}

/// Audio file writer interface (stub).
///
/// Phase 8 will implement WAV writing via `hound`.
/// Input is always planar f64; conversion to target bit depth occurs here.
pub struct AudioFileWriter {
    pub path: String,
    pub n_channels: usize,
    pub sample_rate: f64,
    pub bit_depth: u16,
}

impl AudioFileWriter {
    /// Create an audio file for writing.
    /// TODO: Phase 8 — implement with `hound`.
    pub fn create(
        path: &str,
        n_channels: usize,
        sample_rate: f64,
        bit_depth: u16,
    ) -> Result<Self, FileError> {
        Ok(Self {
            path: path.to_owned(),
            n_channels,
            sample_rate,
            bit_depth,
        })
    }

    /// Write a block of f64 samples.
    pub fn write_block(&mut self, _block: &AudioBlock) -> Result<(), FileError> {
        // TODO: Phase 8 — interleave and convert to target bit depth
        Ok(())
    }

    pub fn finalize(self) -> Result<(), FileError> {
        // TODO: Phase 8 — flush header
        Ok(())
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
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
