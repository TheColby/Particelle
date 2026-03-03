use crate::unit::Unit;

/// Runtime evaluation context passed to every `ParamSignal::eval` call.
///
/// This struct is stack-allocated and must not cause any heap allocation.
pub struct SignalContext<'a> {
    /// Current frame index (monotonic).
    pub frame: u64,
    /// Engine sample rate in Hz.
    pub sample_rate: f64,
    /// Provides runtime field values (MIDI, MPE, control inputs).
    pub fields: &'a dyn FieldProvider,
    /// Optional resolver for custom map functions.
    pub custom_resolver: Option<&'a dyn CustomResolver>,
}

impl<'a> SignalContext<'a> {
    pub fn resolve_custom_map(&self, name: &str, v: f64) -> f64 {
        self.custom_resolver
            .map(|r| r.resolve(name, v))
            .unwrap_or(v)
    }
}

/// A trait for resolving custom mapping functions in the signal graph.
pub trait CustomResolver: Send + Sync {
    fn resolve(&self, name: &str, v: f64) -> f64;
}

/// Provides named field values to the signal graph.
///
/// Implementations are typically populated by the MIDI/MPE routing layer.
/// Field lookup must not allocate.
pub trait FieldProvider: Send + Sync {
    fn get(&self, path: &str) -> Option<f64>;
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> { None }
}

/// A `FieldProvider` that always returns `None`. Useful in tests and offline stubs.
pub struct NullFields;

impl FieldProvider for NullFields {
    fn get(&self, _path: &str) -> Option<f64> {
        None
    }
}

/// Unit-aware field value.
#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub value: f64,
    pub unit: Unit,
}
