/// Metadata for a single output channel.
///
/// Stores the physical/virtual position of the speaker in 3D Cartesian space.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMeta {
    pub name: String,
    pub position: crate::spatializer::Vec3,
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
            ChannelMeta { name: "L".into(), position: crate::spatializer::Vec3::from_az_el(-30.0, 0.0) },
            ChannelMeta { name: "R".into(), position: crate::spatializer::Vec3::from_az_el(30.0, 0.0) },
        ])
    }

    /// Convenience constructor for mono.
    pub fn mono() -> Self {
        Self::new(vec![
            ChannelMeta { name: "M".into(), position: crate::spatializer::Vec3::from_az_el(0.0, 0.0) },
        ])
    }
}
