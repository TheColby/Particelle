/// The position of a speaker in space.
#[derive(Debug, Clone, PartialEq)]
pub enum SpeakerPosition {
    Spherical { azimuth_deg: f64, elevation_deg: f64 },
    Cartesian { x: f64, y: f64, z: f64 },
}

impl SpeakerPosition {
    /// Compute the equivalent 3D Cartesian vector
    pub fn to_vec3(&self) -> crate::spatializer::Vec3 {
        match self {
            Self::Spherical { azimuth_deg, elevation_deg } => {
                crate::spatializer::Vec3::from_az_el(*azimuth_deg, *elevation_deg)
            }
            Self::Cartesian { x, y, z } => {
                crate::spatializer::Vec3::new(*x, *y, *z)
            }
        }
    }
}

/// Metadata for a single output channel.
///
/// Stores the physical/virtual position of the speaker.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMeta {
    pub name: String,
    pub position: SpeakerPosition,
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
            ChannelMeta { name: "L".into(), position: SpeakerPosition::Spherical { azimuth_deg: -30.0, elevation_deg: 0.0 } },
            ChannelMeta { name: "R".into(), position: SpeakerPosition::Spherical { azimuth_deg:  30.0, elevation_deg: 0.0 } },
        ])
    }

    /// Convenience constructor for mono.
    pub fn mono() -> Self {
        Self::new(vec![
            ChannelMeta { name: "M".into(), position: SpeakerPosition::Spherical { azimuth_deg: 0.0, elevation_deg: 0.0 } },
        ])
    }
}
