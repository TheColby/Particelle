use crate::layout::AudioLayout;

/// A point in 3D listener-centric space.
///
/// Coordinates are in meters. +X = right, +Y = forward, +Z = up.
/// The listener is at the origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ORIGIN: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn distance_sq(&self, other: &Vec3) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
}

/// Distributes a signal source positioned in 3D space across output channels.
///
/// Implementations are responsible for computing per-channel gain values from
/// a 3D position, a width parameter, and the known speaker layout. No stereo
/// shortcuts are permitted; all distribution is derived from the layout geometry.
pub trait Spatializer: Send + Sync {
    /// Compute per-channel gains for a source at `position` with the given `width`.
    ///
    /// `out_gains` must have length equal to the layout's channel count.
    fn distribute(&self, position: Vec3, width: f64, out_gains: &mut [f64]);

    /// The layout this spatializer was constructed for.
    fn layout(&self) -> &AudioLayout;
}

/// Amplitude panning spatializer.
///
/// Uses azimuth/elevation proximity to the source position to compute gains.
/// This is the default implementation; the `Spatializer` trait is open to
/// VBAP, ambisonics, or HRTF implementations.
pub struct AmplitudePanner {
    layout: AudioLayout,
}

impl AmplitudePanner {
    pub fn new(layout: AudioLayout) -> Self {
        Self { layout }
    }
}

impl Spatializer for AmplitudePanner {
    fn distribute(&self, _position: Vec3, _width: f64, out_gains: &mut [f64]) {
        // TODO: implement azimuth/elevation weighted gain distribution
        // Placeholder: equal power across all channels
        let n = out_gains.len().min(self.layout.n_channels());
        let equal = 1.0 / (n as f64).sqrt();
        for g in out_gains[..n].iter_mut() {
            *g = equal;
        }
    }

    fn layout(&self) -> &AudioLayout {
        &self.layout
    }
}
