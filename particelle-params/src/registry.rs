use crate::signal::ParamSignal;
use crate::unit::Unit;
use std::collections::HashMap;

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
        self.signals
            .insert(path.clone(), ParamSignal::Const(descriptor.default));
        self.descriptors.insert(path, descriptor);
        Ok(())
    }

    /// Bind a `ParamSignal` to a registered parameter path.
    pub fn bind(&mut self, path: &str, signal: ParamSignal) -> Result<(), RegistryError> {
        if !self.descriptors.contains_key(path) {
            return Err(RegistryError::NotFound {
                path: path.to_owned(),
            });
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
