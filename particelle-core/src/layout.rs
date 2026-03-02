/// Metadata for a single output channel.
///
/// Azimuth is measured in degrees, 0° = front, positive = clockwise.
/// Elevation is measured in degrees, 0° = ear level, 90° = directly above.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMeta {
    pub name: String,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
}

/// Declarative multichannel audio layout.
///
/// Defines the speaker or output channel arrangement without imposing any
/// geometry constraints. Layouts may be stereo, 5.1, 7.1.4, or arbitrary
/// discrete configurations.
#[derive(Debug, Clone)]
pub struct AudioLayout {
    pub channels: Vec<ChannelMeta>,
}

impl AudioLayout {
    pub fn new(channels: Vec<ChannelMeta>) -> Self {
        Self { channels }
    }

    pub fn n_channels(&self) -> usize {
        self.channels.len()
    }

    /// Convenience constructor for stereo (L/R at ±30°).
    pub fn stereo() -> Self {
        Self::new(vec![
            ChannelMeta { name: "L".into(), azimuth_deg: -30.0, elevation_deg: 0.0 },
            ChannelMeta { name: "R".into(), azimuth_deg:  30.0, elevation_deg: 0.0 },
        ])
    }

    /// Convenience constructor for mono.
    pub fn mono() -> Self {
        Self::new(vec![
            ChannelMeta { name: "M".into(), azimuth_deg: 0.0, elevation_deg: 0.0 },
        ])
    }
}
