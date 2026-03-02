use thiserror::Error;
use crate::config::{ParticelleConfig, TuningConfig};

/// Typed validation errors returned by the schema validator.
///
/// The validator never panics. All constraint violations are accumulated and
/// returned as a Vec. An empty Vec means the configuration is valid.
#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    #[error("Field '{field}' is required but missing")]
    MissingRequired { field: String },

    #[error("Field '{field}': value {value} is out of range [{min}, {max}]")]
    OutOfRange { field: String, value: f64, min: f64, max: f64 },

    #[error("Layout has no channels defined")]
    EmptyLayout,

    #[error("Cloud '{id}': unknown window type '{kind}'")]
    UnknownWindowType { id: String, kind: String },

    #[error("Cloud '{id}': invalid source path '{path}'")]
    InvalidSourcePath { id: String, path: String },

    #[error("EDO steps must be > 0")]
    InvalidEdoSteps,

    #[error("JI ratios list is empty")]
    EmptyJiRatios,

    #[error("JI ratio for degree {degree}: denominator must be > 0")]
    InvalidJiDenominator { degree: u32 },

    #[error("Scala: scl_path is empty")]
    EmptyScalaPath,

    #[error("Binding #{index}: target '{target}' is not a registered parameter")]
    UnknownRoutingTarget { index: usize, target: String },
}

/// Known valid window type strings.
const KNOWN_WINDOW_TYPES: &[&str] = &[
    "rectangular", "hann", "hamming", "blackman", "blackman_harris", "nuttall",
    "flat_top", "bartlett", "welch", "parzen", "sine", "power_sine", "tukey",
    "planck_taper", "gaussian", "kaiser", "kbd", "dolph_chebyshev", "dpss",
    "poisson", "exponential", "hann_poisson", "bartlett_hann", "hann_squared",
    "hann_cubed", "rife_vincent", "generalized_blackman", "generalized_cosine",
    "trapezoid", "half_hann", "half_blackman", "asymmetric_tukey", "lanczos",
    "cosine_sum",
];

/// Validate a `ParticelleConfig` and return all constraint violations.
///
/// An empty return Vec means the configuration is internally consistent.
/// The validator is non-destructive; the configuration is not modified.
pub fn validate(config: &ParticelleConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // --- Engine ---
    if config.engine.sample_rate <= 0.0 {
        errors.push(ValidationError::OutOfRange {
            field: "engine.sample_rate".into(),
            value: config.engine.sample_rate,
            min: 1.0,
            max: f64::MAX,
        });
    }
    if config.engine.block_size == 0 {
        errors.push(ValidationError::OutOfRange {
            field: "engine.block_size".into(),
            value: 0.0,
            min: 1.0,
            max: f64::MAX,
        });
    }

    // --- Layout ---
    if config.layout.channels.is_empty() {
        errors.push(ValidationError::EmptyLayout);
    }

    // --- Tuning ---
    match &config.tuning {
        TuningConfig::TwelveTet => {}
        TuningConfig::Edo { steps } => {
            if *steps == 0 {
                errors.push(ValidationError::InvalidEdoSteps);
            }
        }
        TuningConfig::Ji { ratios } => {
            if ratios.is_empty() {
                errors.push(ValidationError::EmptyJiRatios);
            }
            for r in ratios {
                if r.den == 0 {
                    errors.push(ValidationError::InvalidJiDenominator { degree: r.degree });
                }
            }
        }
        TuningConfig::Scala { scl_path, .. } => {
            if scl_path.is_empty() {
                errors.push(ValidationError::EmptyScalaPath);
            }
        }
    }

    // --- Clouds ---
    for cloud in &config.clouds {
        let kind = cloud.window.kind.to_lowercase();
        if !KNOWN_WINDOW_TYPES.contains(&kind.as_str()) {
            errors.push(ValidationError::UnknownWindowType {
                id: cloud.id.clone(),
                kind: cloud.window.kind.clone(),
            });
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    fn minimal_config() -> ParticelleConfig {
        ParticelleConfig {
            engine: EngineConfig {
                sample_rate: 48000.0,
                block_size: 256,
                max_particles_per_cloud: 4096,
            },
            hardware: None,
            layout: LayoutConfig {
                channels: vec![
                    ChannelConfig::Spherical { name: "L".into(), azimuth_deg: -30.0, elevation_deg: 0.0 },
                    ChannelConfig::Spherical { name: "R".into(), azimuth_deg:  30.0, elevation_deg: 0.0 },
                ],
            },
            tuning: TuningConfig::TwelveTet,
            clouds: vec![],
            routing: RoutingConfig::default(),
        }
    }

    #[test]
    fn valid_config_has_no_errors() {
        let cfg = minimal_config();
        assert!(validate(&cfg).is_empty());
    }

    #[test]
    fn zero_sample_rate_errors() {
        let mut cfg = minimal_config();
        cfg.engine.sample_rate = 0.0;
        let errors = validate(&cfg);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::OutOfRange { .. })));
    }

    #[test]
    fn empty_layout_errors() {
        let mut cfg = minimal_config();
        cfg.layout.channels.clear();
        assert!(validate(&cfg).iter().any(|e| matches!(e, ValidationError::EmptyLayout)));
    }

    #[test]
    fn zero_edo_steps_errors() {
        let mut cfg = minimal_config();
        cfg.tuning = TuningConfig::Edo { steps: 0 };
        assert!(validate(&cfg).iter().any(|e| matches!(e, ValidationError::InvalidEdoSteps)));
    }
}
