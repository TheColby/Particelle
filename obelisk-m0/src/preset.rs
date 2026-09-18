//! Versioned, portable M0 preset state.

use crate::{M0Config, ZoneConfig};
use serde::{Deserialize, Serialize};

pub const M0_PRESET_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct M0Preset {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub parameters: M0PresetParameters,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct M0PresetParameters {
    pub base_morph: f32,
    #[serde(default)]
    pub modulation_hz: f32,
    #[serde(default)]
    pub delay_mix: f32,
    #[serde(default)]
    pub reverb_mix: f32,
}

impl Default for M0PresetParameters {
    fn default() -> Self {
        Self {
            base_morph: 0.5,
            modulation_hz: 0.0,
            delay_mix: 0.0,
            reverb_mix: 0.0,
        }
    }
}

impl M0Preset {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        parameters: M0PresetParameters,
    ) -> Self {
        Self {
            schema_version: M0_PRESET_SCHEMA_VERSION,
            id: id.into(),
            name: name.into(),
            tags: Vec::new(),
            parameters,
        }
    }

    pub fn apply_to(&self, config: &mut M0Config) {
        config.base_morph = self.parameters.base_morph;
        config.modulation_hz = self.parameters.modulation_hz;
        config.delay_mix = self.parameters.delay_mix;
        config.reverb_mix = self.parameters.reverb_mix;
    }

    pub fn validate(&self) -> Result<(), PresetError> {
        if self.schema_version != M0_PRESET_SCHEMA_VERSION {
            return Err(PresetError::UnsupportedSchema(self.schema_version));
        }
        let mut config = M0Config {
            zone: ZoneConfig::lower(15),
            ..M0Config::default()
        };
        self.apply_to(&mut config);
        config.validate().map_err(PresetError::InvalidParameters)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PresetError {
    #[error("unsupported M0 preset schema version {0}")]
    UnsupportedSchema(u32),
    #[error("invalid preset parameters: {0}")]
    InvalidParameters(crate::M0EngineError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_round_trip_and_apply() {
        let preset = M0Preset::new(
            "glass",
            "Glass",
            M0PresetParameters {
                base_morph: 0.8,
                modulation_hz: 0.2,
                delay_mix: 0.3,
                reverb_mix: 0.4,
            },
        );
        let parsed: M0Preset =
            serde_json::from_str(&serde_json::to_string(&preset).unwrap()).unwrap();
        parsed.validate().unwrap();
        let mut config = M0Config::default();
        parsed.apply_to(&mut config);
        assert_eq!(config.reverb_mix, 0.4);
    }
}
