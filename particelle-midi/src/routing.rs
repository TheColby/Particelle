use crate::events::MidiEvent;
use particelle_params::registry::ParamRegistry;

/// Routing rule: maps a MIDI source path to a parameter path.
#[derive(Debug, Clone)]
pub struct RoutingRule {
    /// Source identifier, e.g. `"midi.cc.1"`, `"mpe.pressure"`, `"mpe.pitchbend"`.
    pub source: String,
    /// Target parameter path, e.g. `"cloud.shimmer.density"`.
    pub target: String,
    /// Optional affine transform: `value = input * scale + offset`.
    pub scale: f64,
    pub offset: f64,
}

impl RoutingRule {
    pub fn direct(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            scale: 1.0,
            offset: 0.0,
        }
    }
}

/// Routes incoming MIDI events to Field values in the signal graph.
///
/// Applied off the audio thread. Results are written into a field map that
/// the audio thread reads via the `FieldProvider` interface.
pub struct MidiRouter {
    pub rules: Vec<RoutingRule>,
}

impl MidiRouter {
    pub fn new(rules: Vec<RoutingRule>) -> Self {
        Self { rules }
    }

    /// Validate all routing targets exist in the registry.
    pub fn validate(&self, registry: &ParamRegistry) -> Vec<String> {
        self.rules.iter()
            .filter(|r| registry.get_descriptor(&r.target).is_none())
            .map(|r| format!("Routing target not found: '{}'", r.target))
            .collect()
    }

    /// Process a batch of MIDI events and write field values.
    /// TODO: Phase 6 — write into a lock-free field store.
    pub fn process(&self, _events: &[MidiEvent]) {
        // TODO: parse source paths, apply transforms, write to field store
    }
}
