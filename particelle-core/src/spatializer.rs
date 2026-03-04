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
    fn distribute(&self, position: Vec3, orientation: Vec3, directivity: f64, width: f64, out_gains: &mut [f64]);

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
    fn distribute(&self, position: Vec3, orientation: Vec3, directivity: f64, width: f64, out_gains: &mut [f64]) {
        let n = out_gains.len().min(self.speaker_vecs.len());
        let source_dir = position.normalize();
        
        // Radiation gain (Anisotropic Directivity)
        // Vector from grain to listener (origin)
        let to_listener = Vec3::new(-source_dir.x, -source_dir.y, -source_dir.z);
        let radiation_dot = orientation.dot(&to_listener);
        let radiation_gain = (directivity + (1.0 - directivity) * radiation_dot).max(0.0);
        
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
        
        // Constant power normalization AND radiation attenuation
        let norm = if sum_sq > f64::EPSILON { 1.0 / sum_sq.sqrt() } else { 0.0 };
        for g in out_gains[..n].iter_mut() {
            *g *= norm * radiation_gain;
        }
    }

    fn layout(&self) -> &AudioLayout {
        &self.layout
    }
}

/// Binaural (HRTF) spatializer based on a spherical head model.
///
/// Computes Interaural Intensity Differences (IID) based on acoustic head shadowing.
/// Assumes a rigid sphere radius $a$ where shadowing occurs as a function of the
/// incident angle to each ear. Currently frequency-independent (broadband approximation).
pub struct HrtfSpatializer {
    layout: AudioLayout,
    left_ear: Vec3,
    right_ear: Vec3,
}

impl HrtfSpatializer {
    pub fn new(layout: AudioLayout) -> Self {
        // Assert stereo layout for HRTF
        assert_eq!(layout.channels.len(), 2, "HRTF Spatializer requires exactly 2 output channels.");
        
        // Define human ears ideally at -90 (L) and +90 (R) degrees azimuth.
        Self {
            layout,
            left_ear: Vec3::from_az_el(-90.0, 0.0),
            right_ear: Vec3::from_az_el(90.0, 0.0),
        }
    }
    
    /// Computes the broadband spherical head shadow gain for a given incident angle.
    /// Uses a continuous approximation of the shadow zone.
    fn head_shadow(incident_dot: f64) -> f64 {
        // incident_dot is cos(theta) where theta is angle to ear.
        // 1.0 = looking directly at the ear.
        // -1.0 = looking at the opposite ear.
        
        // Smooth transition from full gain on ipsilateral side to shadowed on contralateral.
        // Scale to a realistic IID max depth (e.g., ~15-20dB shadow = ~0.1 amplitude).
        let min_gain = 0.15; // approximate max head shadow attenuation
        let alpha = (incident_dot + 1.0) / 2.0; // map [-1, 1] -> [0, 1]
        
        // Non-linear shadow curve
        min_gain + (1.0 - min_gain) * alpha.powf(1.5)
    }
}

impl Spatializer for HrtfSpatializer {
    fn distribute(&self, position: Vec3, orientation: Vec3, directivity: f64, _width: f64, out_gains: &mut [f64]) {
        if out_gains.len() < 2 { return; }
        
        let source_dir = position.normalize();
        
        // Radiation gain (Anisotropic Directivity)
        // Vector from grain to listener (origin)
        let to_listener = Vec3::new(-source_dir.x, -source_dir.y, -source_dir.z);
        let radiation_dot = orientation.dot(&to_listener);
        let radiation_gain = (directivity + (1.0 - directivity) * radiation_dot).max(0.0);
        
        // Dot product gives cosine of angle between source and ear
        let l_dot = source_dir.dot(&self.left_ear);
        let r_dot = source_dir.dot(&self.right_ear);
        
        // Compute shadowed gains
        let mut l_gain = Self::head_shadow(l_dot);
        let mut r_gain = Self::head_shadow(r_dot);
        
        // Constant power normalize
        let sum_sq = l_gain * l_gain + r_gain * r_gain;
        if sum_sq > f64::EPSILON {
            let norm = 1.0 / sum_sq.sqrt();
            l_gain *= norm;
            r_gain *= norm;
        } else {
            l_gain = std::f64::consts::FRAC_1_SQRT_2;
            r_gain = std::f64::consts::FRAC_1_SQRT_2;
        }
        
        out_gains[0] = l_gain * radiation_gain;
        out_gains[1] = r_gain * radiation_gain;
    }

    fn layout(&self) -> &AudioLayout {
        &self.layout
    }
}

/// Higher-Order Ambisonic (HOA) Encoder up to 3rd order.
///
/// Encodes a 3D point source into Ambisonic B-format using ACN channel
/// ordering and SN3D (Schmidt Semi-Normalized) spherical harmonics.
/// The number of required channels is `(order + 1)^2`. Max order = 3 (16 channels).
pub struct AmbisonicEncoder {
    layout: AudioLayout,
    order: usize,
    channels: usize,
}

impl AmbisonicEncoder {
    pub fn new(layout: AudioLayout) -> Self {
        let channels = layout.channels.len();
        // Determine order based on channel count: N = (order + 1)^2
        let order = match channels {
            1 => 0,
            4 => 1,
            9 => 2,
            16 => 3,
            _ => panic!("Ambisonic layout must have 1, 4, 9, or 16 channels for 0th-3rd order. Got {}.", channels),
        };
        
        Self { layout, order, channels }
    }
}

impl Spatializer for AmbisonicEncoder {
    fn distribute(&self, position: Vec3, orientation: Vec3, directivity: f64, _width: f64, out_gains: &mut [f64]) {
        if out_gains.len() < self.channels { return; }
        
        // Normalize position to unit vector
        let v = position.normalize();
        let x = v.x;
        let y = v.y;
        let z = v.z;
        
        // Radiation gain (Anisotropic Directivity)
        let to_listener = Vec3::new(-x, -y, -z);
        let radiation_dot = orientation.dot(&to_listener);
        let radiation_gain = (directivity + (1.0 - directivity) * radiation_dot).max(0.0);
        
        // Compute SN3D Spherical Harmonics up to 3rd Order incrementally
        // ACN Ordering (0 to 15)
        
        // Order 0
        out_gains[0] = 1.0 * radiation_gain; // W
        
        // Order 1
        if self.order >= 1 {
            out_gains[1] = y * radiation_gain; // Y
            out_gains[2] = z * radiation_gain; // Z
            out_gains[3] = x * radiation_gain; // X
        }
        
        // Order 2
        if self.order >= 2 {
            let root3 = 3.0f64.sqrt();
            let root3_2 = root3 / 2.0;

            out_gains[4] = root3 * x * y * radiation_gain;                            // V
            out_gains[5] = root3 * y * z * radiation_gain;                            // T
            out_gains[6] = 0.5 * (3.0 * z * z - 1.0) * radiation_gain;                // R
            out_gains[7] = root3 * x * z * radiation_gain;                            // S
            out_gains[8] = root3_2 * (x * x - y * y) * radiation_gain;                // U
        }
        
        // Order 3
        if self.order >= 3 {
            let root15 = 15.0f64.sqrt();
            let root5_8 = (5.0 / 8.0_f64).sqrt();

            out_gains[9]  = root5_8 * y * (3.0 * x * x - y * y) * radiation_gain;     // Q
            out_gains[10] = root15 * x * y * z * radiation_gain;                      // O
            out_gains[11] = root5_8 * y * (5.0 * z * z - 1.0) * radiation_gain;       // M
            out_gains[12] = 0.5 * z * (5.0 * z * z - 3.0) * radiation_gain;           // K
            out_gains[13] = root5_8 * x * (5.0 * z * z - 1.0) * radiation_gain;       // L
            out_gains[14] = root15 / 2.0 * z * (x * x - y * y) * radiation_gain;      // N
            out_gains[15] = root5_8 * x * (x * x - 3.0 * y * y) * radiation_gain;     // P
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
        
        panner.distribute(Vec3::from_az_el(-30.0, 0.0), Vec3::from_az_el(0.0, 0.0), 1.0, 0.1, &mut gains);
        assert!(gains[0] > gains[1]); // Left speaker louder than right

        panner.distribute(Vec3::from_az_el(0.0, 0.0), Vec3::from_az_el(0.0, 0.0), 1.0, 0.1, &mut gains);
        assert!((gains[0] - gains[1]).abs() < 1e-10); // Center is equal
    }

    #[test]
    fn test_hrtf_spatializer() {
        let layout = AudioLayout::stereo();
        let panner = HrtfSpatializer::new(layout);
        let mut gains = vec![0.0; 2];
        
        // Source heavily to the left
        panner.distribute(Vec3::from_az_el(-90.0, 0.0), Vec3::from_az_el(0.0, 0.0), 1.0, 0.1, &mut gains);
        assert!(gains[0] > gains[1]); // Left ear receives significantly more gain
        assert!(gains[1] > 0.1);      // Right ear still receives acoustic shadow (not absolute 0)

        // Source dead center
        panner.distribute(Vec3::from_az_el(0.0, 0.0), Vec3::from_az_el(0.0, 0.0), 1.0, 0.1, &mut gains);
        assert!((gains[0] - gains[1]).abs() < 1e-10); // Perfectly balanced
    }

    #[test]
    fn test_ambisonic_encoder_1st_order() {
        // Create 4-channel layout for 1st-order AmbiX
        let mut chs = Vec::new();
        for i in 0..4 {
            chs.push(crate::layout::ChannelMeta {
                name: format!("ACN{}", i),
                position: crate::layout::SpeakerPosition::Cartesian { x: 0.0, y: 0.0, z: 0.0 }
            });
        }
        let layout = AudioLayout { channels: chs };
        let encoder = AmbisonicEncoder::new(layout);
        
        let mut gains = vec![0.0; 4];
        
        // Encode source at front (y = 1)
        encoder.distribute(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 1.0, 0.1, &mut gains);
        assert_eq!(gains[0], 1.0); // W is always 1.0 in SN3D
        assert_eq!(gains[1], 1.0); // Y = y = 1.0
        assert_eq!(gains[2], 0.0); // Z = z = 0.0
        assert_eq!(gains[3], 0.0); // X = x = 0.0
        
        // Encode source at right (x = 1)
        encoder.distribute(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 1.0, 0.1, &mut gains);
        assert_eq!(gains[0], 1.0); // W
        assert_eq!(gains[1], 0.0); // Y
        assert_eq!(gains[2], 0.0); // Z
        assert_eq!(gains[3], 1.0); // X
    }
}
