use std::collections::HashMap;
use crate::signal::ParamSignal;
use crate::unit::Unit;

/// Describes the domain type of a parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    Continuous,
    Discrete,
    Boolean,
}

/// Registered metadata for a single parameter.
#[derive(Debug, Clone)]
pub struct ParamDescriptor {
    /// Dotted canonical path: e.g. `"cloud.shimmer.density"`.
    pub path: String,
    pub unit: Unit,
    pub domain: Domain,
    /// Valid range [min, max].
    pub range: (f64, f64),
    /// Default value (in native units).
    pub default: f64,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Registry mapping canonical parameter paths to their descriptors and signals.
///
/// Every parameter that exists in the engine must be registered here before
/// it can be read or modulated. No unregistered parameter may be modulated.
#[derive(Default)]
pub struct ParamRegistry {
    descriptors: HashMap<String, ParamDescriptor>,
    signals: HashMap<String, ParamSignal>,
}

impl ParamRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a parameter descriptor. Returns an error if already registered.
    pub fn register(&mut self, descriptor: ParamDescriptor) -> Result<(), RegistryError> {
        let path = descriptor.path.clone();
        if self.descriptors.contains_key(&path) {
            return Err(RegistryError::AlreadyRegistered { path });
        }
        self.signals.insert(path.clone(), ParamSignal::Const(descriptor.default));
        self.descriptors.insert(path, descriptor);
        Ok(())
    }

    /// Bind a `ParamSignal` to a registered parameter path.
    pub fn bind(&mut self, path: &str, signal: ParamSignal) -> Result<(), RegistryError> {
        if !self.descriptors.contains_key(path) {
            return Err(RegistryError::NotFound { path: path.to_owned() });
        }
        self.signals.insert(path.to_owned(), signal);
        Ok(())
    }

    pub fn get_descriptor(&self, path: &str) -> Option<&ParamDescriptor> {
        self.descriptors.get(path)
    }

    pub fn get_signal(&self, path: &str) -> Option<&ParamSignal> {
        self.signals.get(path)
    }

    pub fn all_paths(&self) -> impl Iterator<Item = &str> {
        self.descriptors.keys().map(|s| s.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Parameter already registered: '{path}'")]
    AlreadyRegistered { path: String },
    #[error("Parameter not found: '{path}'")]
    NotFound { path: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_success() {
        let mut registry = ParamRegistry::new();
        let descriptor = ParamDescriptor {
            path: "test.param".to_string(),
            unit: Unit::Normalized,
            domain: Domain::Continuous,
            range: (0.0, 1.0),
            default: 0.5,
            description: None,
        };

        let result = registry.register(descriptor.clone());
        assert!(result.is_ok());

        let registered_descriptor = registry.get_descriptor("test.param");
        assert!(registered_descriptor.is_some());
        assert_eq!(registered_descriptor.unwrap().path, "test.param");
        assert_eq!(registered_descriptor.unwrap().default, 0.5);

        let registered_signal = registry.get_signal("test.param");
        assert!(registered_signal.is_some());
        if let ParamSignal::Const(val) = registered_signal.unwrap() {
            assert_eq!(*val, 0.5);
        } else {
            panic!("Expected Const signal");
        }
    }

    #[test]
    fn test_register_already_registered() {
        let mut registry = ParamRegistry::new();
        let descriptor = ParamDescriptor {
            path: "test.param".to_string(),
            unit: Unit::Normalized,
            domain: Domain::Continuous,
            range: (0.0, 1.0),
            default: 0.5,
            description: None,
        };

        let result1 = registry.register(descriptor.clone());
        assert!(result1.is_ok());

        let result2 = registry.register(descriptor);
        assert!(result2.is_err());
        match result2.unwrap_err() {
            RegistryError::AlreadyRegistered { path } => {
                assert_eq!(path, "test.param");
            }
            _ => panic!("Expected AlreadyRegistered error"),
        }
    }
}
