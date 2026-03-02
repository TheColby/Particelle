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

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > f64::EPSILON {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            Self::FORWARD
        }
    }

    pub fn from_az_el(azimuth_deg: f64, elevation_deg: f64) -> Self {
        let az = azimuth_deg.to_radians();
        let el = elevation_deg.to_radians();
        let cos_el = el.cos();
        Self {
            x: cos_el * az.sin(),
            y: cos_el * az.cos(),
            z: el.sin(),
        }
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
    speaker_vecs: Vec<Vec3>,
}

impl AmplitudePanner {
    pub fn new(layout: AudioLayout) -> Self {
        let speaker_vecs = layout.channels.iter()
            .map(|ch| ch.position.to_vec3().normalize())
            .collect();
        Self { layout, speaker_vecs }
    }
}

impl Spatializer for AmplitudePanner {
    fn distribute(&self, position: Vec3, width: f64, out_gains: &mut [f64]) {
        let n = out_gains.len().min(self.speaker_vecs.len());
        let source_dir = position.normalize();
        
        // Spread exponent controlled by width
        let p = if width >= 0.99 { 0.0 } else { 1.0 / (width.max(0.01) * 2.0) };
        
        let mut sum_sq = 0.0;
        for i in 0..n {
            let dot = source_dir.dot(&self.speaker_vecs[i]);
            // Only the hemisphere facing the speaker receives gain
            let gain = if dot > 0.0 { dot.powf(p) } else { 0.0 };
            out_gains[i] = gain;
            sum_sq += gain * gain;
        }
        
        // Constant power normalization
        let norm = if sum_sq > f64::EPSILON { 1.0 / sum_sq.sqrt() } else { 0.0 };
        for g in out_gains[..n].iter_mut() {
            *g *= norm;
        }
    }

    fn layout(&self) -> &AudioLayout {
        &self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_from_az_el() {
        let front = Vec3::from_az_el(0.0, 0.0);
        assert!((front.y - 1.0).abs() < 1e-10);

        let right = Vec3::from_az_el(90.0, 0.0);
        assert!((right.x - 1.0).abs() < 1e-10);

        let up = Vec3::from_az_el(0.0, 90.0);
        assert!((up.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_amplitude_panner_stereo() {
        let layout = AudioLayout::stereo();
        let panner = AmplitudePanner::new(layout);
        let mut gains = vec![0.0; 2];
        
        panner.distribute(Vec3::from_az_el(-30.0, 0.0), 0.1, &mut gains);
        assert!(gains[0] > gains[1]); // Left speaker louder than right

        panner.distribute(Vec3::from_az_el(0.0, 0.0), 0.1, &mut gains);
        assert!((gains[0] - gains[1]).abs() < 1e-10); // Center is equal
    }
}
