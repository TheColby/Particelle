use particelle_core::FrameCount;
use crate::unit::Unit;

/// Runtime evaluation context passed to every `ParamSignal::eval` call.
///
/// This struct is stack-allocated and must not cause any heap allocation.
pub struct SignalContext<'a> {
    /// Current frame index (monotonic).
    pub frame: FrameCount,
    /// Engine sample rate in Hz.
    pub sample_rate: f64,
    /// Provides runtime field values (MIDI, MPE, control inputs).
    pub fields: &'a dyn FieldProvider,
}

/// Provides named field values to the signal graph.
///
/// Implementations are typically populated by the MIDI/MPE routing layer.
/// Field lookup must not allocate.
pub trait FieldProvider: Send + Sync {
    fn get(&self, path: &str) -> Option<f64>;
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
